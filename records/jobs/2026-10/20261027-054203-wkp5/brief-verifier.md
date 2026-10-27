# Job 20261027-054203-wkp5: verify "Make planner name and confirm each demonstration's test seam"

## Request (verbatim)

> Priority: normal
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

Root cause: the planning stage lets a demonstration call code that cannot be driven that way (e.g. a fake injected into a function that builds its own client), because neither the planner prompt, the planner brief nor the critic requires the seam to be named and checked in the code. Fix it in planner.md, critic.md and the brief text in briefs.rs, with a test that fails without it. This addresses the plan-critique findings that stopped jobs 20261018-095726-r3bn, 20261018-113512-gvcs and 20261024-115052-bzw6, all of the same class, by preventing the class rather than redoing those jobs.

Assumptions:
- Goal: planners stop writing demonstrations that assume a nonexistent seam; audience is the planner and critic agents and the maintainer judging planning-round failures.
- Fix lives in prompt text (crates/athena-context/prompts/planner.md and critic.md) and the brief built in crates/athena-context/src/briefs.rs, not in the Demonstration record schema, to keep the change small and avoid a records schema bump.
- Job 20261018-095726-r3bn findings (criterion 1 seam for Athena::land_pr, replay assertion, signs_commits, dead_code claim, gh.rs scope) are instances of the class; they no longer apply to this change because it edits the planning stage, not land/gh code. Job 20261018-113512-gvcs findings (cut in a test double, replay range, guard preconditions) likewise. Job 20261024-115052-bzw6 findings (Athena::land_pr wiring untested; fake Runner possible) likewise; the earlier jobs' out-of-scope item on planning-stage changes is superseded by this request.
- The rule asks, per demonstration: name the production entry point and its preconditions or guards, the fake/seam/fixture and how it is injected, and whether it exists today or which step adds it; logic under test must not live in the test double.
- Tests are in-crate unit tests in briefs.rs checking the rendered planner and critic text.

## Acceptance criteria

1. [content] planner.md requires each demonstration to name the production entry point it calls, its guards and preconditions, the seam, fake or fixture and how it is injected, and whether the seam exists today (checked by reading the code) or which plan step adds it; it forbids putting the logic under test inside the test double.
2. [content] critic.md tells the critic to raise a major finding when a demonstration names no seam, or names one the code does not provide, or tests only a double.
3. [content] The planner brief rendered by planner_brief repeats the seam requirement in its Report section, so the rule is enforced in the brief and not only in the prompt.
4. [behavior] A new unit test in crates/athena-context asserts the planner brief and the planner and critic prompts contain the seam requirement; run `cargo test -p athena-context`. It fails on the base commit and passes with the change.
5. [behavior] `cargo fmt --check`, `cargo clippy` and `cargo test` pass.
6. [content] The change description names the root cause (the planning stage never requires a demonstration to name and confirm its test seam) and that it is fixed in the planning stage, not at one job's instance.

## Expected deliverables

- crates/athena-context/prompts/planner.md (modify)
- crates/athena-context/prompts/critic.md (modify)
- crates/athena-context/src/briefs.rs (modify)
- CHANGELOG.md (modify)
- crates/athena-cli/tests/job_plan.rs (modify)

## Out of scope

- Changing the three-round cap or the critic's verdict logic
- Adding fields to the Demonstration record or bumping its schema
- Redoing the landing, records or PR changes from the referenced jobs
- Changing CI workflows or governance files

## User defaults that apply

- root-cause: Fix the root cause, not the symptom or the one instance that surfaced. The fault is fixed where it originates; when that lies outside the job's scope, name it and propose the fix rather than working around it. The change description names the root cause and why the change addresses it. The root-cause fix is the one focused change. Here: Fix in the planning stage where the missing requirement originates.
- tested: Every module has passing tests; cargo test passes and clippy is clean. Here: Add a unit test; fmt, clippy, test pass.
- behavior-evidence: Acceptance criteria must check the requested behavior happening, not that types, files or wiring exist. A protection or denial test first shows the action succeeds without the protection, as the same user and in the same environment the real component runs as (root), then shows the action is refused. It fails, not skips or passes silently, if the environment cannot enforce the protection; a file mode that root bypasses is not proof. Worked examples are in docs/design/defaults-loop.md section 3.7. Here: Test checks rendered brief text contains the requirement and fails without it.
- small: Prefer one focused change; size is a design choice, not a wish list. Here: Prompt and brief text only, no schema change.
- mechanism: Rules that matter are enforced by mechanism, or explicitly marked prose-only. Here: Rule placed in the generated brief plus a test, not prose alone.
- changelog: (not in the defaults model) Here: BRIEF.md asks for tagged releases and a changelog, so the change records itself in CHANGELOG.md.

## Scope

The expected deliverables list is the one statement of scope. A changed file that is not on that list is the scope finding, judged against the list and not against the wording of a criterion.

## The change

Base f6fe878fb776 → candidate d1cf26b000d6. In your working directory, `git diff HEAD~1` shows it. records/, the owner's private files, and history are absent by design; their absence is not a finding.

```text
 CHANGELOG.md                             | 10 ++++++
 crates/athena-cli/tests/job_plan.rs      |  5 +++
 crates/athena-context/prompts/critic.md  |  1 +
 crates/athena-context/prompts/planner.md |  1 +
 crates/athena-context/src/briefs.rs      | 52 +++++++++++++++++++++++++++++++-
 crates/athena-context/src/lib.rs         |  2 +-
 6 files changed, 69 insertions(+), 2 deletions(-)
```

(The diff was inlined for the verifier; it is elided in this record.)

## Checks Athena ran on this tree

- fmt: `cargo fmt --all --check` passed
- clippy: `cargo clippy --all-targets --locked -- -D warnings` passed
- test: `cargo test --locked` failed (pre-existing: this check already failed on the base commit)
- criteria-left-to-athena-checks: `not a check` passed (the producer left these criteria to Athena's recorded checks (fmt, clippy, the full test suite); judge them yourself from the recorded check results: `cargo fmt --check`, `cargo clippy` and `cargo test` pass.)

## Commands Bash may run

- cargo build | check | test | clippy | fmt | run | metadata | tree
- git status | diff | log | show
- ls (paths inside your working directory)

## Verdict

End with the verdict as your structured output: one criteria entry per acceptance criterion, in order.
