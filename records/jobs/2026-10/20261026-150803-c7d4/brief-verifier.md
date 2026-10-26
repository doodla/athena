# Job 20261026-150803-c7d4: verify "Resume planning-capped jobs from the kept plan; report planning cost per"

## Request (verbatim)

> Priority: urgent
> A plan that stops at the three-round cap with only a small fix left throws away the whole planning spend, and the retry starts over from a fresh reading and plan. Today this happened three times (20261026-091156-d0k9, 20261025-001236-brde, 20261026-145955-s7e2): each round-3 verdict was "revise" with one major finding the critic itself called small, plus minors. Make a planning stop resumable from its last plan: when planning stops at the cap, keep the round-3 plan and the critic's findings with the job, and have `athena retry` of a planning-stopped job start the planner from that plan plus the findings (one revision round, then the critic) instead of a fresh plan. Measure the effect: report in athena status the planning rounds and planning cost spent per landed job, before and after, from the job records.
> Acceptance:
> - A test stops a job at the planning cap with a fixture critic; `athena retry` of it gives the planner the kept plan and the critic findings, and the first critic round of the retry sees the revised plan.
> - A test shows a retry of a job that stopped for any other reason still starts with a fresh plan.
> - athena status shows planning rounds and planning cost per landed job, computed from records.
> - fmt, clippy with -D warnings and the test suite pass; CHANGELOG.md has an entry.
> Files: crates/athena-core/src/run.rs, crates/athena-core/src/reading.rs, crates/athena-core/src/status.rs, crates/athena-cli/src/display.rs, CHANGELOG.md

## Reading (accepted by the user)

When planning stops at the three-round cap, keep the round-3 plan and the critic's findings with the job. `athena retry` of such a job seeds the planner with them (one revision round, then the critic) instead of a fresh plan. Retries of jobs stopped for any other reason still plan fresh. `athena status` reports planning rounds and planning cost per landed job from records. Audience: the Athena operator.

Assumptions:
- Goal: stop wasting planning spend when a plan hits the round cap with only small findings left. Audience: the Athena operator.
- 'Before and after' in status is read as a split of landed jobs by whether the planning-resume change applied. The split is a cutover point derived from records, and the values are labeled as measured.
- Kept plan and findings may need a new typed record in athena-data; if existing records already hold the plan and critique, no new variant is added.
- If the retried plan's base or request changed, resume still applies, since only planning-stopped jobs qualify.
- The default retry behavior for non-planning stops is unchanged.

## Acceptance criteria

1. [behavior] A test stops a job at the planning cap with a fixture critic. `athena retry` of it gives the planner the kept round-3 plan and the critic's findings. The first critic round of the retry sees the revised plan, and the retry does only one revision round before the critic. Run: cargo test --locked (named test).
2. [behavior] A test shows a retry of a job that stopped for any other reason (e.g. timeout, verification failure) still starts the planner with a fresh plan and receives no kept plan. Run: cargo test --locked.
3. [behavior] athena status prints planning rounds and planning cost per landed job, computed from job records. A test with fixture records asserts the printed numbers, including a before/after split. Run: cargo test --locked.
4. [content] The kept plan and findings are stored with the job as records, with no secrets or personal data.
5. [behavior] cargo fmt --all --check && cargo clippy --all-targets --locked -- -D warnings && cargo test --locked all exit 0.
6. [content] CHANGELOG.md has an entry describing resumable planning stops and the new status measure.

## Expected deliverables

- crates/athena-core/src/run.rs (modify)
- crates/athena-core/src/reading.rs (modify)
- crates/athena-core/src/status.rs (modify)
- crates/athena-cli/src/display.rs (modify)
- CHANGELOG.md (modify)
- crates/athena-context/src/briefs.rs (modify)
- crates/athena-data/src/records.rs (modify)
- crates/athena-cli/tests/lifecycle.rs (modify)

## Out of scope

- Changing the planning round cap
- Changing how the critic grades findings
- Retry behavior for non-planning stops
- Backfilling records for old jobs beyond computing the measure from existing records

## User defaults that apply

- tested: Every module has passing tests; cargo test passes and clippy is clean. Here: New tests for resume, fresh retry, and status; fmt, clippy -D warnings and test suite must pass.
- behavior-evidence: Acceptance criteria must check the requested behavior happening, not that types, files or wiring exist. A protection or denial test first shows the action succeeds without the protection, as the same user and in the same environment the real component runs as (root), then shows the action is refused. It fails, not skips or passes silently, if the environment cannot enforce the protection; a file mode that root bypasses is not proof. Worked examples are in docs/design/defaults-loop.md section 3.7. Here: Criteria check the planner input and the critic's first-round input, not just that fields exist.
- outcome-check: The outcome is the check behind the work, not the activity done. For audience-facing quality, a fresh model reader that took no part in the work rates the live result as a newcomer, and a rating below 4 of 5 blocks completion. Prose-only: no loop code computes the rating yet. Here: Status measure reports rounds and cost per landed job from records.
- small: Prefer one focused change; size is a design choice, not a wish list. Here: One focused change in the listed files.
- rust: All code is Rust. Non-Rust files appear only where a platform requires them, and each is listed with its reason. Here: All code is Rust.
- public: Everything committed is public: no secrets, credentials, or personal data, and transcripts are redacted. Here: Kept plans contain no secrets.
- changelog: (not in the defaults model) Here: BRIEF.md asks for tagged releases and a changelog, so the change records itself in CHANGELOG.md.

## Scope

The expected deliverables list is the one statement of scope. A changed file that is not on that list is the scope finding, judged against the list and not against the wording of a criterion.


## Planned demonstrations (round 3)

A critic accepted these demonstrations before the change was made. Judge the request and the criteria first, and run each demonstration that applies to a behavior criterion. A changed file that was not planned, or a planned file that did not change, is a finding only when it leaves a criterion unshown or adds scope.

- Criterion 1: Integration test in lifecycle.rs. A Scenario's critic returns Revise in all three rounds, so `go` stops at the cap. Then `athena.retry` and `go` run on the new job, with the script wrapped to record each spec's role and prompt. Run: `cargo test --locked --test lifecycle a_retry_of_a_job_stopped_at_the_planning_cap_resumes_from_the_kept_plan`. Observes: The first retry planner prompt contains the kept round-3 approach and every round-3 finding. After the retry's reader the calls are exactly Planner then Critic. The first retry critic prompt contains the revised approach and not the kept one. The retry's job-plan-1.json has `resumed_from` equal to the old job.
  Fails without the behavior: Without the seed the retry's first planner prompt is a fresh brief with no kept approach or findings, so the contains assertions fail. The call order and the critic's prompt would also differ.
- Criterion 2: Integration tests in lifecycle.rs retry a job that stopped on a plan reject, one that stopped on a producer timeout (`deadline_script`), and one that failed verification. A reading.rs unit test checks `planning_resume_seed` is None for these and for a non-retry request. Run: `cargo test --locked a_retry_of_a_job_that_stopped_for_any_other_reason_plans_fresh`. Observes: Each retry's first planner prompt has no revision section, no kept-plan text and no findings. Its job-plan-1.json has no `resumed_from`.
  Fails without the behavior: If resume applied to every retry or to any planning stop, the reject case would show the kept plan in the prompt and the absence assertions would fail.
- Criterion 3: status.rs unit tests with hand-built Records. Before group: a cap-stopped predecessor plus a landed retry that planned fresh. After group: a cap-stopped predecessor plus a landed retry whose round-1 plan has `resumed_from`. An unlanded job and an unplanned landed job must be excluded. A display.rs render test checks the printed text. A third test checks the 'after' group is empty when no plan is resumed. Run: `cargo test --locked planning_cost`. Observes: Chain-total mean rounds and mean USD match the fixture, with predecessors included, and the own figures sit beside them. Group counts and the cutover job id and time are right. The text shows both groups and the label 'since the first resumed planning'.
  Fails without the behavior: With no planning-cost computation there are no such lines and the numeric assertions fail. A per-job-only computation fails the predecessor-included totals.
- Criterion 4: The same lifecycle test loads the retry's committed records. The kept plan and findings are the old job's job-plan-3 and critique-3. The retry's job-plan-1 carries only a job id in `resumed_from`. A records.rs round-trip test checks that old records without the field still parse. Run: `cargo test --locked a_retry_of_a_job_stopped_at_the_planning_cap_resumes_from_the_kept_plan`. Observes: Records load and `athena.check_history()` reports no violations. `resumed_from` holds only a job id. Fixture text has no secrets or personal data.
  Fails without the behavior: Without the marker the `resumed_from` assertion fails. Without the optional-field default, the old-record round-trip test in records.rs fails.
- Criterion 5: Run the three Athena checks as one command chain. Run: `cargo fmt --all --check && cargo clippy --all-targets --locked -- -D warnings && cargo test --locked`. Observes: All three exit 0.
  Fails without the behavior: The new tests (resume, fresh retry, planning_cost, render, round-trip) are part of the suite. If the behavior or its tests were missing, or the field edits left store.rs or admission.rs not compiling, clippy or test would exit non-zero. Unformatted code fails fmt.
- Criterion 6: Read the Unreleased section of the diff. Run: `git diff -- CHANGELOG.md`. Observes: A new entry describes the resumable planning stop (kept plan and findings; retry runs one revision round then the critic). It also describes planning rounds and cost per landed job in status as chain totals, with 'after' meaning 'since the first resumed planning'.

Planned files:

- crates/athena-data/src/records.rs (modify)
- crates/athena-context/src/briefs.rs (modify)
- crates/athena-core/src/reading.rs (modify)
- crates/athena-core/src/run.rs (modify)
- crates/athena-core/src/status.rs (modify)
- crates/athena-cli/src/display.rs (modify)
- crates/athena-cli/tests/lifecycle.rs (modify)
- CHANGELOG.md (modify)
- crates/athena-core/src/store.rs (modify)
- crates/athena-core/src/admission.rs (modify)

## The change

Base 0e762f3039b3 → candidate 5c1928472675. In your working directory, `git diff HEAD~1` shows it. records/, the owner's private files, and history are absent by design; their absence is not a finding.

```text
 CHANGELOG.md                         |  13 +
 crates/athena-cli/src/display.rs     |  85 ++++++-
 crates/athena-cli/tests/lifecycle.rs | 282 ++++++++++++++++++++++
 crates/athena-context/src/briefs.rs  |  45 +++-
 crates/athena-core/src/admission.rs  |   1 +
 crates/athena-core/src/reading.rs    | 143 +++++++++++
 crates/athena-core/src/run.rs        |  20 +-
 crates/athena-core/src/status.rs     | 445 ++++++++++++++++++++++++++++++++++-
 crates/athena-core/src/store.rs      |   1 +
 crates/athena-data/src/records.rs    |  25 ++
 10 files changed, 1049 insertions(+), 11 deletions(-)
```

(The diff was inlined for the verifier; it is elided in this record.)

## Checks Athena ran on this tree

- fmt: `cargo fmt --all --check` passed
- clippy: `cargo clippy --all-targets --locked -- -D warnings` passed
- test: `cargo test --locked` passed (1346 tests ran)
- criteria-left-to-athena-checks: `not a check` passed (the producer left these criteria to Athena's recorded checks (fmt, clippy, the full test suite); judge them yourself from the recorded check results: cargo fmt --all --check && cargo clippy --all-targets --locked -- -D warnings && cargo test --locked all exit 0.)

## Commands Bash may run

- cargo build | check | test | clippy | fmt | run | metadata | tree
- git status | diff | log | show
- ls (paths inside your working directory)

## Verdict

End with the verdict as your structured output: one criteria entry per acceptance criterion, in order.
