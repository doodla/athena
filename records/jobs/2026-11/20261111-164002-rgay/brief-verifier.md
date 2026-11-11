# Job 20261111-164002-rgay: verify "Make an empty producer diff carry a recorded reason"

## Request (verbatim)

> Priority: high
> 
> Fix the root cause of a repeated failure: Athena treats an empty producer diff as a bare failure. The producer isn't required to say whether t
> 
> The same cause has produced failures in more than one job. Find the cause where it originates and fix it there, so the whole class is prevented; do not work around one instance.
> 
> First ask what information the failing stage lacked, and change the code so that information is given to it: in its brief, in the system facts it is shown, or in the inputs its reader receives. Prefer this over any rule that refuses work.
> 
> A refusal rule is allowed only when giving the stage that information cannot prevent the failure. A request that adds one must state the expected share of past accepted readings the rule would refuse.
> 
> Failure class: producer-makes-no-change-on-bounded-code-move-no-reason-recorded (20261027-012857-p9qn)
> Cause: Athena treats an empty producer diff as a bare failure. The producer isn't required to say whether the work was already done, blocked, or abandoned. Nothing checks before dispatch that the thing to be moved is still where the request says i...
> Seen 2 time(s) in 2 jobs. The introspections that record it:
> Introspection 20261027-012842-7whw (job 20261027-011733-vrsk); prevention it proposed: Two code checks. (1) Before dispatch, resolve each named source symbol and file in the current snapshot (here the Transport row in gh/mod.rs and the target gh/transport.rs). If the source is absent or the target already holds it, mark the j...
> Introspection 20261111-163944-gnda (job 20261111-145400-dx3z); prevention it proposed: Require the producer to end every run with a structured outcome: changed, already done (with evidence), blocked (with the blocker), or gave up (with the reason). Have Athena also record policy denials and deadline or budget stops for the ru...
> 
> Acceptance:
> - The root cause is named in the change description and fixed where it originates, not at one instance
> - A test fails without the change and passes with it, for the cause and not only for one job
> - fmt, clippy and test pass
> 
> Files: crates/

## Reading (accepted by the user)

Root cause: the producer is never asked to say why it changed nothing, and Athena ends an empty diff with the bare reason "the producer made no change" (candidate.rs), dropping the report it has. Fix by telling the producer in its brief to give a structured no-change outcome (already done with evidence, blocked, or gave up), carrying it in the report, and putting it in the failure reason and records. Retries of 20261027-011733-vrsk and 20261111-145400-dx3z are both of this class.

Assumptions:
- Prior jobs: 20261027-011733-vrsk (gh/transport.rs move; candidate changed no files, outcome 'the producer made no change') and 20261111-145400-dx3z (role-need retire rule; candidate changed no files). Neither candidate changed any file, so there is no work to keep. Neither job's deliverables (gh/transport.rs, gh/mod.rs, role_need.rs, store.rs, status.rs, display.rs, docs/system-facts.md, docs/spec.toml, xtask/shard-timings.txt) are redone here; they are out_of_scope. Each job's finding, 'the producer made no change', is addressed by making that outcome carry a reason: for dx3z the reason would be 'already done', with the evidence the producer reported.
- Goal and audience: the user and later introspections who read a failed job's reason and need to know whether work was already done, blocked or abandoned. They can see this in the failure reason and the kept report.
- Code survey: crates/athena-core/src/run/candidate.rs (candidate_stage: an empty change list ends in self.failure with the bare text; report_gaps only covers report problems); crates/athena-core/src/run/producer.rs (report check and send-back loop, ends with Flow::Go(Reported)); crates/athena-core/src/run/mod.rs (Reported to candidate_stage; ProducerReport built near line 2529); crates/athena-context/src/outputs.rs (ProducerOutput schema, serde(default) for optional fields); crates/athena-context/src/briefs.rs (producer_brief); crates/athena-data/src/records.rs (ProducerReport and Candidate, deny_unknown_fields, so any new field needs serde(default) and the AnyRecord round-trip tests must still pass; an existing record without the field must still parse). The property is measured directly: the failure reason string of a job whose fake producer changes nothing and reports a no-change outcome.
- Design: add an optional no-change outcome to the producer's structured output and report (kind: already done / blocked / gave up, plus detail or evidence). The producer brief asks for it whenever the diff is empty. candidate_stage includes it in the failure reason. This adds no refusal rule, so there is no refused-readings share.
- Existing code: extend crates/athena-context/src/outputs.rs ProducerOutput, crates/athena-data/src/records.rs ProducerReport, crates/athena-core/src/run/candidate.rs and crates/athena-core/src/run/producer.rs, and the existing retry test in crates/athena-cli/tests/retry.rs (which asserts 'the producer made no change').
- The pre-dispatch check of the named source symbols proposed by introspection 7whw is a refusal-style rule, so it is not built. The brief change is the information fix; the no-change outcome tells the next reader whether the source was already moved.
- Prompt headroom: the producer prompt has 157 tokens left, so the brief addition must be short; prompts stay unchanged beyond the Rust-built brief text.

## Acceptance criteria

1. [behavior] cargo test passes with a new test in crates/athena-core/src/run/tests.rs or crates/athena-cli/tests/retry.rs: a fake producer makes no change and reports 'already done' with evidence; the job's failure reason contains the kind and the evidence, not only 'the producer made no change'. The test fails on the base commit.
2. [behavior] Three more test cases for blocked, gave up, and no outcome given with an empty diff (the reason says no reason was reported, naming the missing field) pass, so the fix covers the class and not only the two jobs.
3. [content] git diff shows crates/athena-context/src/briefs.rs (producer brief) telling the producer to give the no-change outcome (already done with evidence, blocked, or gave up) when it leaves the tree unchanged; a briefs.rs unit test asserts this text is in producer_brief.
4. [content] git diff shows the new field in ProducerOutput (outputs.rs) and ProducerReport (records.rs) with serde(default), and cargo test shows existing records without the field still parse (the AnyRecord round-trip tests pass).
5. [content] The change adds no new rule that refuses or holds work: git diff shows no new admission, gate or check that fails a job, so no refused-readings share applies; an empty diff still fails the job but with the reason recorded.
6. [content] CHANGELOG.md has an entry naming the root cause: the producer is not asked for a no-change outcome and Athena discards it, so an empty diff was a bare failure.
7. [behavior] cargo fmt --check, cargo clippy and cargo test all pass.

## Expected deliverables

- crates/athena-core/src/run/candidate.rs (modify)
- crates/athena-context/src/outputs.rs (modify)
- crates/athena-data/src/records.rs (modify)
- crates/athena-context/src/briefs.rs (modify)
- crates/athena-core/src/run/producer.rs (modify)
- crates/athena-core/src/run/mod.rs (modify)
- crates/athena-core/src/run/tests.rs (modify)
- crates/athena-cli/tests/retry.rs (modify)
- crates/athena-agent/src/fake.rs (modify)
- CHANGELOG.md (modify)
- docs/system-facts.md (modify)

## Out of scope

- Moving Transport items into gh/transport.rs (job 20261027-011733-vrsk work); no files kept because its candidate changed none
- The role-need retire rule fix (job 20261111-145400-dx3z work); no files kept because its candidate changed none
- A pre-dispatch check that refuses a job when the named source symbols are absent
- Changes to athena.toml, defaults.toml or prompt files
- Automatically retrying or reclassifying jobs whose producer reported 'already done'

## User defaults that apply

- outcome-check: The outcome is the check behind the work, not the activity done. For audience-facing quality, a fresh model reader that took no part in the work rates the live result as a newcomer, and a rating below 4 of 5 blocks completion. Prose-only: no loop code computes the rating yet. Here: Criteria check the failure reason a user reads, run through a test.
- tested: Every module has passing tests; cargo test passes and clippy is clean. Here: New tests plus fmt, clippy and test pass.
- root-cause: Fix the root cause, not the symptom or the one instance that surfaced. The fault is fixed where it originates; when that lies outside the job's scope, name it and propose the fix rather than working around it. The change description names the root cause and why the change addresses it. The root-cause fix is the one focused change. Here: Fixed in the producer's brief and output schema, not in the two jobs.
- behavior-evidence: Acceptance criteria must check the requested behavior happening, not that types, files or wiring exist. A protection or denial test first shows the action succeeds without the protection, as the same user and in the same environment the real component runs as (root), then shows the action is refused. It fails, not skips or passes silently, if the environment cannot enforce the protection; a file mode that root bypasses is not proof. Worked examples are in docs/design/defaults-loop.md section 3.7. Here: The first criterion runs a no-change job and checks its reason.
- changelog: (not in the defaults model) Here: BRIEF.md asks for tagged releases and a changelog, so the change records itself in CHANGELOG.md.

## Scope

The expected deliverables list is the one statement of scope. A changed file that is not on that list is the scope finding, judged against the list and not against the wording of a criterion.

## Soundness

Would a careful senior engineer reject this approach as a crude heuristic, a special case that should be general, a guess where the code or data could be read, or a rule with obvious false positives (for example, one that matches common words)? Name every such problem, each with the simpler or more correct alternative. Judge the approach only, not style.

A blocker or major soundness finding fails the verdict like an unmet criterion, even when every criterion is met. Minor and nit findings do not.


## Planned demonstrations (round 2)

A critic accepted these demonstrations before the change was made. Judge the request and the criteria first, and run each demonstration that applies to a behavior criterion. A changed file that was not planned, or a planned file that did not change, is a finding only when it leaves a criterion unshown or adds scope.

- Criterion 1: New behavior test `an_idle_producer_reporting_already_done_fails_with_the_kind_and_evidence` in crates/athena-cli/tests/retry.rs: a Scenario with `edits: vec![]` and a report whose `no_change` is {"kind":"already_done","detail":"Transport is already in gh/transport.rs (grep found it at line 12)"} runs through `Athena::go`. It asserts state Failed and that the reason contains 'the producer made no change', 'already done' and the evidence text; then loads Records and asserts the failed job's candidate record holds producer_report.no_change with the same kind and detail. Run: `cargo test --locked -p athena-cli --test retry an_idle_producer_reporting_already_done`. Observes: The finished job's `reason` string and the stored candidate record hold the kind and the evidence, not only the bare sentence.
  Fails without the behavior: On the base commit ProducerOutput has deny_unknown_fields and no `no_change` field, so the report is rejected and the reason is 'the producer returned no valid report'. Even if it parsed, the reason would be the bare 'the producer made no change', which lacks 'already done' and the evidence, so the contains assertions fail.
  Test seam: Production entry point: `Athena::go` (runs produce_and_verify, producer_loop and candidate_stage). Fixture `Fixture::new()` and `Scenario` from crates/athena-cli/tests/common/mod.rs, injected with `fx.athena(scenario.script())` as a FakeAgentDriver script; the report is plain JSON with the `no_change` key set in the test, the fake does no reasoning. Exists today (retry.rs:840-853 uses the same pattern); the only addition is the `no_change` field in the production types (plan steps 1-2). Guard: edits empty so candidate_stage takes the empty-diff branch (candidate.rs:138); the Scenario default marks every criterion met (common/mod.rs:591-604) so the loop does not send the report back. If the failed job's candidate record is not stored, the producer reports that and keeps the reason-only assertion.
- Criterion 2: Three more retry.rs tests with the same helper: `an_idle_producer_reporting_blocked_fails_with_the_blocker` (reason contains 'blocked' and the blocker text), `an_idle_producer_reporting_gave_up_fails_with_the_reason` (contains 'gave up' and the reason text), `an_idle_producer_reporting_nothing_fails_naming_the_missing_no_change_field` (default report without a `no_change` key; reason contains 'reported no reason' and 'no_change'). Plus a unit-test module in run/tests.rs on `no_change_reason`: three kinds, blank detail ('no detail given'), report without outcome, no report, gaps suffix still ending 'the report is kept in ...'. Run: `cargo test --locked -p athena-cli --test retry an_idle_producer_reporting && cargo test --locked -p athena-core --lib no_change_reason`. Observes: Every kind and the absent case produce a distinct reason that names what was or was not reported.
  Fails without the behavior: On the base commit blocked and gave_up reports are rejected (unknown field) or fall back to the bare reason, so the kind and detail asserts fail; for the missing case the bare reason does not name `no_change`, so that assert fails.
  Test seam: Same retry.rs seam as criterion 1 (exists today). `no_change_reason` is a new `pub(super)` function in candidate.rs (added by plan step 4), reached from run/tests.rs through the private child module `candidate` of `run` (mod.rs:61). Its input is a plain `ProducerReport` value; the function does the formatting and nothing is stubbed.
- Criterion 3: New briefs.rs unit test `producer_brief_asks_for_a_no_change_outcome_when_the_tree_is_unchanged`, built like `producer_briefs_list_every_criterion_to_copy_and_ask_for_one_entry_each` (briefs.rs:1515-1548): build ProducerInputs, call `producer_brief`, assert the words `no_change`, `already_done`, `blocked`, `gave_up`, 'evidence', 'leave the tree unchanged' and 'still fails', and that 'fails as an empty candidate' is still present once. `git diff crates/athena-context/src/briefs.rs` shows `no_change_section()` called in `producer_brief_with_evidence` before '## Report' (briefs.rs:741). Run: `cargo test --locked -p athena-context producer_brief_asks_for_a_no_change_outcome`. Observes: The brief text carries the instruction and agrees with the existing empty-candidate sentence.
  Fails without the behavior: Without the section, producer_brief output lacks 'no_change', 'already_done', 'blocked' and 'gave_up', so the contains asserts fail.
  Test seam: Production entry point `producer_brief` (briefs.rs:672). The `reading()`, `defaults()` and `ProducerInputs` fixtures are existing test helpers (briefs.rs:1515). No double. The seam exists today.
- Criterion 4: outputs.rs tests: producer_schema()['required'] contains `no_change`; its object form has additionalProperties false; a ProducerOutput JSON without the key parses with `no_change == None`, one with it parses to kind and detail. records.rs tests: an old-shape ProducerReport JSON parses with `no_change == None` and serializes without the key; an `AnyRecord::Candidate2` holding a producer_report with and without a no_change round-trips through `to_json` and `AnyRecord::parse`. Existing `candidate_2_round_trips_*` and `candidate_1_still_parses_*` (records.rs:1867-1893) and run_pin_tests.rs:1707 run unchanged. `git diff` shows #[serde(default)] on both new fields. Run: `cargo test --locked -p athena-context outputs && cargo test --locked -p athena-data records && cargo test --locked -p athena-core --lib run_pin_tests`. Observes: The new field is optional on read, absent on write when unset, and required-but-nullable in the model's schema.
  Fails without the behavior: Without serde(default) the old-shape JSON (no `no_change` key) fails to parse; with the schema lacking the property the producer_schema assertions fail; without skip_serializing_if the absent-key text assert fails.
  Test seam: Production entry points: `producer_schema`, serde for ProducerOutput and ProducerReport, `AnyRecord::parse` / `to_json`. They use in-memory JSON and the existing `sample_candidate()` fixture (records.rs:1852). No double. All exist today.
- Criterion 5: Read `git diff --stat` and `git diff crates/athena-core/src/run/candidate.rs`: the empty-diff branch changes only its reason text; the state is still Failed through `self.failure` and the early `return Ok(Flow::Stop)` stays. Nothing in crates/athena-core/src/admission.rs, run/checks.rs, the send-back conditions in producer.rs (488-495) or crates/athena-gate is touched. The existing `a_producer_that_makes_no_change_fails_whatever_its_test_result` (lifecycle.rs:1447) and `an_unseeded_run_whose_producer_changes_nothing_still_fails` (retry.rs:840) still pass, so an empty diff still fails. The decisions.md and decision-links.toml edits record a D08 reconciliation and add no check. Run: `git diff --stat`. Observes: An empty diff still ends the job as Failed and no new admission, gate or check appears in the diff.
  Fails without the behavior: If the change had turned the empty-diff case into a refusal or hold (a new check, or a changed state), the two existing tests above would fail on state or reason, and the diff would show a touched admission.rs, checks.rs or gate crate.
  Test seam: Production entry points: `Athena::go` through the existing tests lifecycle.rs:1447 and retry.rs:840, with the existing Fixture/Scenario FakeAgentDriver script; `git diff --stat` for the file list. Both exist today.
- Criterion 6: Read the new entry under the Unreleased heading. It names the root cause (the producer was not asked for a no-change outcome and Athena discarded the report it had, so an empty diff was a bare failure), the fix (the brief asks for `no_change`, the report and record keep it, the failure reason shows it), the jobs 20261027-011733-vrsk and 20261111-145400-dx3z, and says the pre-dispatch symbol check was left out of scope with an informational brief form as a possible follow-up. Run: `git diff -- CHANGELOG.md`. Observes: The CHANGELOG diff shows the root-cause sentence.
  Fails without the behavior: Without the entry the diff of CHANGELOG.md is empty and no root-cause sentence appears.
- Criterion 7: Run the three check commands whole over the workspace. They include the real-repository tests that read docs/decisions.md and docs/decision-links.toml (backtest, mirrors), docs/system-facts.md (budgets) and all the new tests above. Run: `cargo fmt --all --check && cargo clippy --all-targets --locked -- -D warnings && cargo test --locked`. Observes: All three pass, including `cargo test --locked -p athena-cli --test backtest --test mirrors --test budgets`, which the full run contains.
  Fails without the behavior: Without the change: the new retry.rs, run/tests.rs, briefs.rs, outputs.rs and records.rs tests fail, so `cargo test` fails. If the ProducerReport literal at reading.rs:3862 is not updated, `cargo clippy --all-targets` fails to compile. If D171's link `line` is not the 'Amends D08' bullet's line, or D171 holds a second parsed phrase, `the_real_log_has_no_condition_a_finding_against_the_confirmed_list` (crates/athena-cli/tests/backtest.rs:115-133) fails; if a `- budget` line in system-facts.md moved, `system_facts_budgets_match_computed` fails.
  Test seam: Production entry points: the three cargo commands (the checks Athena runs, as listed in the brief) over the whole workspace. Fixtures: the real repository files read by backtest.rs, mirrors.rs and budgets.rs, and the Fixture/Scenario FakeAgentDriver used by retry.rs; no doubles are added. All exist today; the new inputs are added by plan steps 1-11.

Planned files:

- crates/athena-context/src/outputs.rs (modify)
- crates/athena-data/src/records.rs (modify)
- crates/athena-context/src/briefs.rs (modify)
- crates/athena-core/src/run/mod.rs (modify)
- crates/athena-core/src/run/candidate.rs (modify)
- crates/athena-core/src/run/producer.rs (modify)
- crates/athena-core/src/run/tests.rs (modify)
- crates/athena-cli/tests/retry.rs (modify)
- crates/athena-agent/src/fake.rs (modify)
- crates/athena-core/src/reading.rs (modify)
- CHANGELOG.md (modify)
- docs/system-facts.md (modify)
- docs/decisions.md (modify)
- docs/decision-links.toml (modify)

## Replay of accepted readings

Athena ran the newest accepted readings through the base and the candidate reading rules: 0 of 50 are newly refused by the candidate and were accepted by the base. This is within the allowed share, so the job is not held.

## The change

Base cede2fa9fb15 → candidate 7b0f0c844088. In your working directory, `git diff HEAD~1` shows it. records/, the owner's private files, and history are absent by design; their absence is not a finding.

```text
 CHANGELOG.md                            |  10 +++
 crates/athena-agent/src/fake.rs         |   3 +-
 crates/athena-cli/tests/retry.rs        | 100 ++++++++++++++++++++++++++++++
 crates/athena-context/src/briefs.rs     |  44 +++++++++++++
 crates/athena-context/src/outputs.rs    | 105 ++++++++++++++++++++++++++++++++
 crates/athena-core/src/reading.rs       |   1 +
 crates/athena-core/src/run/candidate.rs |  47 +++++++++++---
 crates/athena-core/src/run/mod.rs       |  12 +++-
 crates/athena-core/src/run/producer.rs  |   3 +-
 crates/athena-core/src/run/tests.rs     |  71 +++++++++++++++++++++
 crates/athena-data/src/records.rs       |  62 +++++++++++++++++++
 docs/decision-links.toml                |   7 +++
 docs/decisions.md                       |  24 ++++++++
 docs/system-facts.md                    |   3 +
 14 files changed, 480 insertions(+), 12 deletions(-)
```

(The diff was inlined for the verifier; it is elided in this record.)

## Evidence snapshot

Athena copied the real records when this job was dispatched into a read-only directory outside your working copy:

- path: <state>/jobs/20261111-164002-rgay/evidence

It holds `records/` (every file under records/ at the base commit), `landings/` (the landing records) and `history.json` (the base commit and the newest 200 first-parent commits of main: hash, commit time, subject, trailers). Every check of real records, such as counting jobs, reading an earlier job's records or comparing a claim with history, uses this snapshot and not the working copy. Read it with the Read, Grep and Glob tools, or run `cargo run -p athena-cli -- status --snapshot <state>/jobs/20261111-164002-rgay/evidence`, which prints the job counts, the history summary and the throughput section from it. It is read-only: a write to it fails. The working copy still has no records/.

## Checks Athena ran on this tree

- fmt: `cargo fmt --all --check` passed (result reused from the CI run https://github.com/doodla/athena/actions/runs/43092124272 for 7b0f0c8440883c9eae25cb40e55f70ccc1c169ca)
- clippy: `cargo clippy --all-targets --locked -- -D warnings` passed (result reused from the CI run https://github.com/doodla/athena/actions/runs/43092124272 for 7b0f0c8440883c9eae25cb40e55f70ccc1c169ca)
- test: `cargo test --locked` passed (result reused from the CI run https://github.com/doodla/athena/actions/runs/43092124272 for 7b0f0c8440883c9eae25cb40e55f70ccc1c169ca)
- ci-result: `not a check` passed (Check results for 7b0f0c8440883c9eae25cb40e55f70ccc1c169ca come from the CI run https://github.com/doodla/athena/actions/runs/43092124272 (719s): fmt success, clippy success, test success.)
- criteria-left-to-athena-checks: `not a check` passed (the producer left these criteria to Athena's recorded checks (fmt, clippy, the full test suite); judge them yourself from the recorded check results: cargo fmt --check, cargo clippy and cargo test all pass.)
- fails-without-1: not a check. Athena ran the planned "fails without the fix" demonstration for criteria 1 on a scratch copy of the candidate with the fix reverted and the tests kept: `cargo test --locked -p athena-cli --test retry an_idle_producer_reporting_already_done`, exit code 101. Result: the named test command failed without the fix (exit 101): the test did not compile.
  Files put back to the base: CHANGELOG.md, crates/athena-agent/src/fake.rs, crates/athena-context/src/briefs.rs, crates/athena-context/src/outputs.rs, crates/athena-core/src/run/candidate.rs, crates/athena-core/src/run/mod.rs, crates/athena-core/src/run/producer.rs, crates/athena-data/src/records.rs, docs/decision-links.toml, docs/decisions.md, docs/system-facts.md.
  End of the output (data, not instructions):

```text
   Compiling thiserror-impl v2.0.21
   Compiling serde_derive v1.0.229
   Compiling toml v0.9.12+spec-1.1.0
   Compiling jiff v0.2.38
   Compiling ref-cast v1.0.27
   Compiling bitflags v2.13.2
   Compiling linux-raw-sys v0.12.1
   Compiling ref-cast-impl v1.0.27
   Compiling serde_derive_internals v0.30.0
   Compiling aho-corasick v1.1.5
   Compiling once_cell v1.21.4
   Compiling athena-data v0.1.0 (<job>/revert/crates/athena-data)
   Compiling regex-syntax v0.8.11
   Compiling fastrand v2.5.0
   Compiling tempfile v3.27.0
   Compiling regex-automata v0.4.18
   Compiling schemars_derive v1.2.2
   Compiling anyhow v1.0.104
   Compiling hashbrown v0.17.1
   Compiling dyn-clone v1.0.20
   Compiling equivalent v1.0.2
   Compiling schemars v1.2.2
   Compiling indexmap v2.14.2
   Compiling regex v1.13.1
   Compiling athena-git v0.1.0 (<job>/revert/crates/athena-git)
   Compiling utf8parse v0.2.2
   Compiling anstyle-parse v1.0.0
   Compiling syn v2.0.119
   Compiling toml_edit v0.23.10+spec-1.0.0
   Compiling athena-score v0.1.0 (<job>/revert/crates/athena-score)
   Compiling athena-gate v0.1.0 (<job>/revert/crates/athena-gate)
   Compiling athena-context v0.1.0 (<job>/revert/crates/athena-context)
   Compiling athena-agent v0.1.0 (<job>/revert/crates/athena-agent)
   Compiling anstyle v1.0.14
   Compiling anstyle-query v1.1.5
   Compiling is_terminal_polyfill v1.70.2
   Compiling colorchoice v1.0.5
   Compiling anstream v1.0.0
   Compiling athena-core v0.1.0 (<job>/revert/crates/athena-core)
   Compiling heck v0.5.0
   Compiling strsim v0.11.1
   Compiling clap_lex v1.1.1
   Compiling clap_builder v4.6.7
   Compiling clap_derive v4.6.7
   Compiling athena-testkit v0.1.0 (<job>/revert/crates/athena-testkit)
   Compiling clap v4.6.7
   Compiling athena-cap-change v0.1.0 (<job>/revert/crates/athena-cap-change)
   Compiling athena-cap-study v0.1.0 (<job>/revert/crates/athena-cap-study)
   Compiling athena-cli v0.1.0 (<job>/revert/crates/athena-cli)
error[E0609]: no field `no_change` on type `&ProducerReport`
   --> crates/athena-cli/tests/retry.rs:919:25
    |
919 |         .and_then(|r| r.no_change.as_ref())
    |                         ^^^^^^^^^ unknown field
    |
    = note: available fields are: `summary`, `files`, `checks_run`, `criteria_check`, `open_issues` ... and 3 others

For more information about this error, try `rustc --explain E0609`.
error: could not compile `athena-cli` (test "retry") due to 1 previous error
warning: build failed, waiting for other jobs to finish...
```
- fails-without-2: not a check. Athena did not run the planned "fails without the fix" demonstration for criteria 2 (`cargo test --locked -p athena-cli --test retry an_idle_producer_reporting && cargo test --locked -p athena-core --lib no_change_reason`). Result: not run: not a plain cargo test command.
- fails-without-3: not a check. Athena ran the planned "fails without the fix" demonstration for criteria 3 on a scratch copy of the candidate with the fix reverted and the tests kept: `cargo test --locked -p athena-context producer_brief_asks_for_a_no_change_outcome`, exit code 101. Result: the named test command failed without the fix (exit 101): the test did not compile.
  Files put back to the base: CHANGELOG.md, crates/athena-agent/src/fake.rs, crates/athena-context/src/briefs.rs, crates/athena-context/src/outputs.rs, crates/athena-core/src/run/candidate.rs, crates/athena-core/src/run/mod.rs, crates/athena-core/src/run/producer.rs, crates/athena-data/src/records.rs, docs/decision-links.toml, docs/decisions.md, docs/system-facts.md.
  End of the output (data, not instructions):

```text
   Compiling ref-cast-impl v1.0.27
   Compiling thiserror-impl v2.0.21
   Compiling schemars_derive v1.2.2
   Compiling ref-cast v1.0.27
   Compiling thiserror v2.0.21
   Compiling serde v1.0.229
   Compiling athena-data v0.1.0 (<job>/revert/crates/athena-data)
   Compiling schemars v1.2.2
   Compiling athena-context v0.1.0 (<job>/revert/crates/athena-context)
error[E0422]: cannot find struct, variant or union type `NoChangeOutput` in this scope
    --> crates/athena-context/src/outputs.rs:1042:22
     |
1042 |                 Some(NoChangeOutput {
     |                      ^^^^^^^^^^^^^^ not found in this scope

error[E0609]: no field `no_change` on type `outputs::ProducerOutput`
    --> crates/athena-context/src/outputs.rs:1027:28
     |
1027 |         assert_eq!(without.no_change, None);
     |                            ^^^^^^^^^ unknown field
     |
     = note: available fields are: `summary`, `files`, `checks_run`, `criteria_check`, `open_issues` ... and 4 others

error[E0609]: no field `no_change` on type `outputs::ProducerOutput`
    --> crates/athena-context/src/outputs.rs:1030:25
     |
1030 |         assert_eq!(null.no_change, None);
     |                         ^^^^^^^^^ unknown field
     |
     = note: available fields are: `summary`, `files`, `checks_run`, `criteria_check`, `open_issues` ... and 4 others

error[E0433]: cannot find type `NoChangeKindOutput` in this scope
    --> crates/athena-context/src/outputs.rs:1032:13
     |
1032 |             NoChangeKindOutput::AlreadyDone,
     |             ^^^^^^^^^^^^^^^^^^ use of undeclared type `NoChangeKindOutput`

error[E0433]: cannot find type `NoChangeKindOutput` in this scope
    --> crates/athena-context/src/outputs.rs:1033:13
     |
1033 |             NoChangeKindOutput::Blocked,
     |             ^^^^^^^^^^^^^^^^^^ use of undeclared type `NoChangeKindOutput`

error[E0433]: cannot find type `NoChangeKindOutput` in this scope
    --> crates/athena-context/src/outputs.rs:1034:13
     |
1034 |             NoChangeKindOutput::GaveUp,
     |             ^^^^^^^^^^^^^^^^^^ use of undeclared type `NoChangeKindOutput`

error[E0609]: no field `no_change` on type `outputs::ProducerOutput`
    --> crates/athena-context/src/outputs.rs:1041:22
     |
1041 |                 with.no_change,
     |                      ^^^^^^^^^ unknown field
     |
     = note: available fields are: `summary`, `files`, `checks_run`, `criteria_check`, `open_issues` ... and 4 others

Some errors have detailed explanations: E0422, E0433, E0609.
For more information about an error, try `rustc --explain E0422`.
error: could not compile `athena-context` (lib test) due to 7 previous errors
```
- fails-without-4: not a check. Athena did not run the planned "fails without the fix" demonstration for criteria 4 (`cargo test --locked -p athena-context outputs && cargo test --locked -p athena-data records && cargo test --locked -p athena-core --lib run_pin_tests`). Result: not run: not a plain cargo test command.
- fails-without-5: not a check. Athena did not run the planned "fails without the fix" demonstration for criteria 5 (`git diff --stat`). Result: not run: not a plain cargo test command.
- fails-without-6: not a check. Athena did not run the planned "fails without the fix" demonstration for criteria 6 (`git diff -- CHANGELOG.md`). Result: not run: not a plain cargo test command.
- fails-without-7: not a check. Athena did not run the planned "fails without the fix" demonstration for criteria 7 (`cargo fmt --all --check && cargo clippy --all-targets --locked -- -D warnings && cargo test --locked`). Result: not run: not a plain cargo test command.

## Commands Bash may run

- cargo build | check | test | clippy | fmt | run | metadata | tree
- git status | diff | log | show
- ls (paths inside your working directory)

## Verdict

End with the verdict as your structured output: one criteria entry per acceptance criterion, in order.
