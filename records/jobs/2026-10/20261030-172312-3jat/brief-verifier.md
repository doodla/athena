# Job 20261030-172312-3jat: verify "Make plans name each demonstration's test seam and check it exists"

## Request (verbatim)

> Priority: urgent
> 
> Fix the root cause of a repeated failure: The planning stage writes demonstrations for each criterion without checking that the code under tes
> 
> The same cause has produced failures in more than one job. Find the cause where it originates and fix it there, so the whole class is prevented; do not work around one instance.
> 
> Failure class: plan-demonstration-assumes-nonexistent-test-seam-fails-planning-rounds (20261018-113458-gqky)
> Cause: The planning stage writes demonstrations for each criterion without checking that the code under test can be driven that way. Nothing in the plan template makes it name the seam, fixture or harness each demonstration uses, or confirm that i...
> Seen 5 time(s) in 5 jobs. The introspections that record it:
> Introspection 20261018-113436-cxg9 (job 20261018-095726-r3bn); prevention it proposed: Change the plan template and planning flow in three ways. (1) Each demonstration must name its test seam: the exact function or constructor it calls, how the fake or fixture is injected, and whether that seam exists today or is a planned ch...
> Introspection 20261018-114052-07tm (job 20261018-113512-gvcs); prevention it proposed: Change the plan template and the planning gate. (1) Each demonstration must name its test seam: the fixture or harness, the production function it runs, and whether the seam exists or which step adds it. Logic under test must not sit behind...
> Introspection 20261024-115821-jzkw (job 20261024-115052-bzw6); prevention it proposed: Change the planning stage in three ways. (1) The plan template requires, for each criterion's demonstration, the production entry point called, the fake or seam used, and a check that the seam exists in the code (or the step that adds it). ...
> Introspection 20261025-012621-vezp (job 20261025-001234-2hax); prevention it proposed: Change the planning template and the planning loop. (1) For each criterion the plan must name the production entry point its demonstration calls, that entry point's preconditions, the seam or fake it uses, and the merge method and commit sh...
> Introspection 20261026-024143-4yat (job 20261026-015419-4j37); prevention it proposed: Give the planning stage a required step for each criterion's demonstration. It must name the entry function, quote the state guards and thresholds on the path (such as go refusing Dispatched, or t3_cost needing p90 > 0), and show that the s...
> 
> Acceptance:
> - The root cause is named in the change description and fixed where it originates, not at one instance
> - A test fails without the change and passes with it, for the cause and not only for one job
> - fmt, clippy and test pass
> 
> Files: crates/

## Reading (accepted by the user)

Root cause: a plan demonstration (DemonstrationOutput) has only method, command, observes and fails_without. Nothing makes the planner name the entry point, fake or harness it drives, or whether that seam exists, so critics catch nonexistent seams only in round 3. Fix it at the plan schema and the planning gate: add a required `seam` field for behavior criteria, reject plans without it, and show it to the critic. Jobs 20261018-095726-r3bn, 20261018-113512-gvcs and 20261024-115052-bzw6 stopped at planning round 3 with no candidate. Their code changes are dropped and not redone here; their seam findings are the class this change prevents.

Assumptions:
- Code survey: entry point is crates/athena-core/src/run/planning.rs, where PlanOutput becomes a JobPlan (the function around line 500 that maps DemonstrationOutput to Demonstration), so the new check runs after the planner output parses and before the critic sees the plan. The output schema is crates/athena-context/src/outputs.rs (DemonstrationOutput, line 418), and the critic and planner briefs render demonstrations in crates/athena-context/src/briefs.rs (line 1175).
- Code survey: the stored Demonstration record is in crates/athena-data/src/records.rs (line 2345). The change adds an optional `seam` field there with a serde default, so old plan records still parse. That file is accepted by `check_record_file` in crates/athena-data/src/records.rs, which crates/athena-core/src/run/tests.rs:1117 and crates/athena-cli/tests/lifecycle.rs:71 call, and by the `AnyRecord` round-trip; the new field must pass both, with an old record lacking it still valid.
- Code survey: the property is measured directly by a unit test in crates/athena-core/src/run/tests.rs that calls the production plan-conversion and validation function with a plan whose behavior demonstration has no seam, and one that has it. It does not use a stand-in.
- Audience: the planner, which gets the new required field, and the critic and verifier, who read it. Old stored plans without `seam` still load.
- Seam is required only when a criterion is a behavior criterion (the same condition as fails_without). It states the production entry point, the fake or fixture and how it is injected, and either 'exists' or the plan step that adds it. Whether the seam exists in the code is checked by the critic from the rendered field, not by Athena mechanically.
- Prompt files are out of the crates/ scope given and the planner prompt has room (494 tokens), but the schema field description carries the instruction, so no prompt edit is planned.

## Acceptance criteria

1. [content] The change description names the root cause: plan demonstrations carry no field for the test seam, so the planning flow never requires naming or confirming the seam. Cites jobs 20261018-095726-r3bn, 20261018-113512-gvcs and 20261024-115052-bzw6.
2. [content] DemonstrationOutput in crates/athena-context/src/outputs.rs and Demonstration in crates/athena-data/src/records.rs gain a `seam` field, with a doc comment naming entry point, fake injection and exists-or-added-by-step. The records field has a serde default so old records parse.
3. [content] crates/athena-context/src/briefs.rs renders the seam beside each demonstration, so the critic sees it (check with Grep for the seam label near the Demonstrations: heading).
4. [behavior] Run `cargo test -p athena-core`: a new test shows planning refuses a plan whose behavior-criterion demonstration has an empty or missing seam, with an error naming the criterion, and accepts the same plan with a seam. The test fails on the code before the change.
5. [behavior] Run `cargo test -p athena-data` and `cargo test -p athena-cli`: a plan record without `seam` still passes check_record_file and round-trips as AnyRecord, and a record with the seam round-trips unchanged.
6. [behavior] The change refuses more plans, and the higher refusal rate is intended: expected 10% of newly refused plans, namely behavior criteria with no named seam.
7. [behavior] Run `cargo fmt --check`, `cargo clippy --all-targets -- -D warnings` and `cargo test` from the repository root; all pass.
8. [content] CHANGELOG.md has an entry for the new required seam field and the planning refusal.

## Expected deliverables

- crates/athena-context/src/outputs.rs (modify)
- crates/athena-data/src/records.rs (modify)
- crates/athena-core/src/run/planning.rs (modify)
- crates/athena-core/src/run/tests.rs (modify)
- crates/athena-context/src/briefs.rs (modify)
- CHANGELOG.md (modify)
- crates/athena-core/src/run/mod.rs (modify)
- crates/athena-core/src/status.rs (modify)
- crates/athena-core/src/store.rs (modify)

## Out of scope

- Re-attempting the work of jobs 20261018-095726-r3bn, 20261018-113512-gvcs and 20261024-115052-bzw6 (merge/resync, records-branch landing). Their candidates changed no files, so nothing is kept.
- Changing the three-round planning cap or the critic's severity rules
- Mechanically proving a seam exists in the target code
- Changes to athena.toml, defaults.toml, CI workflows or other governance files
- Edits to records/ or owner files

## User defaults that apply

- rust: All code is Rust. Non-Rust files appear only where a platform requires them, and each is listed with its reason. Here: All changes are Rust.
- tested: Every module has passing tests; cargo test passes and clippy is clean. Here: New tests plus fmt, clippy and test criteria.
- root-cause: Fix the root cause, not the symptom or the one instance that surfaced. The fault is fixed where it originates; when that lies outside the job's scope, name it and propose the fix rather than working around it. The change description names the root cause and why the change addresses it. The root-cause fix is the one focused change. Here: Fixes the plan schema and gate rather than one job's plan.
- behavior-evidence: Acceptance criteria must check the requested behavior happening, not that types, files or wiring exist. A protection or denial test first shows the action succeeds without the protection, as the same user and in the same environment the real component runs as (root), then shows the action is refused. It fails, not skips or passes silently, if the environment cannot enforce the protection; a file mode that root bypasses is not proof. Worked examples are in docs/design/defaults-loop.md section 3.7. Here: Criteria run a test that refuses a seamless plan.
- verified: Done means independently verified, and user-facing behavior is verified by exercising it. Here: Verifier runs cargo test, clippy and fmt.
- mechanism: Rules that matter are enforced by mechanism, or explicitly marked prose-only. Here: The rule is enforced by a validation in planning, not prose.
- small: Prefer one focused change; size is a design choice, not a wish list. Here: One focused schema and gate change.
- changelog: (not in the defaults model) Here: BRIEF.md asks for tagged releases and a changelog, so the change records itself in CHANGELOG.md.

## Scope

The expected deliverables list is the one statement of scope. A changed file that is not on that list is the scope finding, judged against the list and not against the wording of a criterion.

## Soundness

Would a careful senior engineer reject this approach as a crude heuristic, a special case that should be general, a guess where the code or data could be read, or a rule with obvious false positives (for example, one that matches common words)? Name every such problem, each with the simpler or more correct alternative. Judge the approach only, not style.

A blocker or major soundness finding fails the verdict like an unmet criterion, even when every criterion is met. Minor and nit findings do not.


## Planned demonstrations (round 1)

A critic accepted these demonstrations before the change was made. Judge the request and the criteria first, and run each demonstration that applies to a behavior criterion. A changed file that was not planned, or a planned file that did not change, is a finding only when it leaves a criterion unshown or adds scope.

- Criterion 1: Content check of the change description and CHANGELOG: grep for the root-cause wording and the three job ids. Run: `git diff -- CHANGELOG.md`. Observes: The added CHANGELOG entry says plan demonstrations carried no field for the test seam, so planning never required naming or confirming it, and cites 20261018-095726-r3bn, 20261018-113512-gvcs and 20261024-115052-bzw6. The diff is the evidence; the producer's final summary repeats it.
- Criterion 2: Read the diff of outputs.rs and records.rs, then run the record tests so the serde default is exercised. Run: `git diff -- crates/athena-context/src/outputs.rs crates/athena-data/src/records.rs`. Observes: `DemonstrationOutput.seam` and `Demonstration.seam` exist, each with a doc comment naming the production entry point, how the fake or fixture is injected, and 'exists' or the step that adds it. The records field carries `#[serde(default, skip_serializing_if = "Option::is_none")]`. Criterion 5 shows the default at work.
- Criterion 3: Search briefs.rs for the label, and read it beside the `Demonstrations:` heading. A brief unit test also renders a plan with a seam and asserts the line appears. Run: `cargo test -p athena-context seam`. Observes: The test calls the production `critic_brief`/`render_plan_section` path (the briefs.rs test module already builds Demonstration literals at about line 2672, so the fixture exists) with a seam set. It asserts the rendered text contains `Test seam: <text>` under the Demonstrations section, and no seam line when the seam is None. Grep for `Test seam:` in briefs.rs shows the label in `demonstration_lines`, the function the `Demonstrations:` heading calls.
- Criterion 4: Two tests in athena-core. (a) Unit test in admission.rs tests: build a JobPlan literal (the `findings_for_path` helper there shows how) and a `reading()` with one Behavior criterion. Call `plan_findings` with a verifier Policy whose command list allows `cargo test`. (b) End-to-end test in run/tests.rs using `repo_with_planning`, `run_planned` and `planning_script` (all exist today). The reader output is rewritten so criterion 1 is `behavior` and the plan has a `cargo test` demonstration with a fails_without; the producer and verifier stay on the base fake script. Run: `cargo test -p athena-core seam`. Observes: Unit test: a behavior demonstration with seam None, `""` or `"   "` gives a Major Athena finding at location `demonstrations, criterion 1` whose summary names 'criterion 1' and 'seam'. The same plan with a seam gives no seam finding. A content criterion with no seam gives none. End-to-end test: a planner queue of [plan without seam, plan with seam] shows the second planner prompt containing the seam finding and no critic call before it; a queue of only the seamless plan stops with a reason containing 'after 2 mechanical revisions' and the criterion, and never calls the critic, producer or verifier. The end-to-end test asserts planner and critic roles and prompts, not Landed (a behavior reading may not verify with the fake verifier).
  Fails without the behavior: Before the change `plan_findings` has no seam rule. The seamless plan yields no Major seam finding, so the unit assertions fail; in the end-to-end test the critic is called on round 1 (the critic queue is empty or the roles differ) and the stop reason is missing. The `seam` field also does not exist, so the with-seam plan does not compile or parse.
- Criterion 5: Record tests in athena-data (records.rs tests, extending `sample_plan` and the `resumed_from` round-trip pattern) and the existing cli lifecycle path that calls `check_record_file`. Run: `cargo test -p athena-data && cargo test -p athena-cli`. Observes: New athena-data test: the JSON of a plan whose Demonstration has seam None contains no `seam` key, parses via `AnyRecord::parse` to an equal value, and passes `check_record_file` at `<job>/job-plan-<round>.json`. A hand-written old record JSON without the key parses with seam None. A plan with `seam: Some(..)` serializes with the key and round-trips equal. athena-cli tests (lifecycle.rs:71 calls check_record_file; the planned-job tests in job_plan.rs and lifecycle.rs) pass with the updated fixtures, which now carry a seam on their behavior demonstrations.
  Fails without the behavior: Without the serde default and skip-if-none, the old-record JSON fails to parse (`deny_unknown_fields` and a required field) and the none case writes `"seam": null`, so the byte-equal assertion fails. Without the fixture updates, the cli planned-job tests stop at the new Major finding.
- Criterion 6: The same athena-core test table shows the refusal rate is limited to the intended class. It tests only plans for the three cases. The 10% share is an estimate of how often planners omit a seam, not measured here. Run: `cargo test -p athena-core seam`. Observes: Table cases: behavior + no seam is refused; behavior + seam is accepted; content + no seam is accepted; behavior + whitespace seam is refused. Only the first and last are refused, so the new refusal is exactly 'behavior criteria with no named seam'. Nothing else in the plan rules changes: the existing athena-core plan tests still pass.
  Fails without the behavior: Without the gate the refused cases are accepted. If the gate were too wide, the content-criterion case would be refused and the test would fail.
- Criterion 7: Run the full checks from the repository root. Run: `cargo fmt --all --check && cargo clippy --all-targets --locked -- -D warnings && cargo test --locked`. Observes: All three exit 0. Compiling the whole workspace covers every Demonstration literal (records.rs sample_plan, store.rs:536, briefs.rs tests, planning.rs plan_record, admission tests) and the fixtures in athena-agent/src/fake.rs, athena-cli job_plan.rs, lifecycle.rs and introspection.rs. The athena-context outputs.rs schema test (about line 592) and the budget tests still pass.
  Fails without the behavior: Missing the new field in any struct literal fails the build. A fixture without a seam on a behavior criterion fails the planned-job tests.
- Criterion 8: Content check of CHANGELOG.md under `## [Unreleased]`. Run: `git diff -- CHANGELOG.md`. Observes: A new entry under Unreleased (a `### Added` or `### Changed` heading, matching the file's Keep a Changelog form) describes the required `seam` field, the planning refusal for a behavior criterion without a seam, the critic seeing it, and that old records still load.

Planned files:

- crates/athena-context/src/outputs.rs (modify)
- crates/athena-data/src/records.rs (modify)
- crates/athena-core/src/admission.rs (modify)
- crates/athena-core/src/run/planning.rs (modify)
- crates/athena-core/src/run/tests.rs (modify)
- crates/athena-context/src/briefs.rs (modify)
- crates/athena-core/src/store.rs (modify)
- crates/athena-agent/src/fake.rs (modify)
- crates/athena-cli/tests/job_plan.rs (modify)
- crates/athena-cli/tests/lifecycle.rs (modify)
- crates/athena-cli/tests/introspection.rs (modify)
- CHANGELOG.md (modify)

## Replay of accepted readings

Athena ran the newest accepted readings through the base and the candidate reading rules: 0 of 50 are newly refused by the candidate and were accepted by the base. This is within 5 points of the declared share, so the job is not held.

Declared share: 10% of newly refused readings (the job is held above the declared share plus 5 points).
Reason given for the declared share (judge whether it explains why past readings should be refused): CHANGELOG.md has an entry for the new required seam field and the planning refusal.

## The change

Base 02d423375152 → candidate 2b9649b95a95. In your working directory, `git diff HEAD~1` shows it. records/, the owner's private files, and history are absent by design; their absence is not a finding.

```text
 CHANGELOG.md                             |  11 +++
 crates/athena-agent/src/fake.rs          |   3 +-
 crates/athena-cli/tests/introspection.rs |   6 +-
 crates/athena-cli/tests/job_plan.rs      |   3 +-
 crates/athena-cli/tests/lifecycle.rs     |   3 +-
 crates/athena-context/src/briefs.rs      |  22 ++++++
 crates/athena-context/src/outputs.rs     |   6 +-
 crates/athena-core/src/admission.rs      |  78 ++++++++++++++++++++++
 crates/athena-core/src/run/planning.rs   |   1 +
 crates/athena-core/src/run/tests.rs      | 111 ++++++++++++++++++++++++++++++-
 crates/athena-core/src/store.rs          |   1 +
 crates/athena-data/src/records.rs        |  35 ++++++++++
 12 files changed, 273 insertions(+), 7 deletions(-)
```

(The diff was inlined for the verifier; it is elided in this record.)

## Checks Athena ran on this tree

- fmt: `cargo fmt --all --check` passed
- clippy: `cargo clippy --all-targets --locked -- -D warnings` passed
- test: `cargo test --locked` passed (1786 tests ran)
- criteria-left-to-athena-checks: `not a check` passed (the producer left these criteria to Athena's recorded checks (fmt, clippy, the full test suite); judge them yourself from the recorded check results: Run `cargo fmt --check`, `cargo clippy --all-targets -- -D warnings` and `cargo test` from the repository root; all pass.)

## Commands Bash may run

- cargo build | check | test | clippy | fmt | run | metadata | tree
- git status | diff | log | show
- ls (paths inside your working directory)

## Verdict

End with the verdict as your structured output: one criteria entry per acceptance criterion, in order.
