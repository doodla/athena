# Job 20261027-170130-y5fn: verify "Require planner demos to name a verified test seam; keep budgets"

## Request (verbatim)

> try again on 20261027-054203-wkp5

## Reading (accepted by the user)

Retry of job 20261027-054203-wkp5. Fix the root cause (the planning stage never requires a demonstration to name and confirm its test seam) in planner.md, critic.md and the planner brief in briefs.rs, and this time update docs/system-facts.md budget lines and the lib.rs frame-size guard so budget tests pass, with the producer citing the commands it ran.

Assumptions:
- Goal: planner and critic prompts and the planner brief make each demonstration name its seam; audience is the planning stage and the verifier of this change.
- Job 20261027-054203-wkp5 findings: (1) budgets tests failed because docs/system-facts.md planner/critic lines were stale: fixed by recomputing and updating those two lines (never the budget); docs/system-facts.md is now a deliverable and named in a criterion. (2) The lib.rs frame-size threshold edit was outside the deliverables: now listed, and the producer should prefer shortening wording to fit the existing 500-token guard, raising it only if justified in CHANGELOG. (3) Missing command evidence: the producer's report must cite cargo test, clippy and fmt commands run with results.
- Producer first measures remaining planner (1300) and critic (1300) room, adds the seam text, and pays for it by shortening existing wording of equal meaning where needed; budgets themselves are not raised.
- Sizes are ceil(bytes/3); facts lines must equal the computed values exactly, which the budgets test checks.

## Acceptance criteria

1. [content] planner.md requires each demonstration to name the production entry point it calls, its guards and preconditions, the seam/fake/fixture and how it is injected, and whether the seam exists today (checked by reading the code) or which plan step adds it; it forbids putting the logic under test inside the test double.
2. [content] critic.md tells the critic to raise a major finding when a demonstration names no seam, names one the code does not provide, or tests only a double.
3. [content] The planner brief rendered by planner_brief in briefs.rs repeats the seam requirement in its Report section.
4. [content] docs/system-facts.md budget planner and budget critic lines state the computed sizes of the changed prompts, budgets (of 1300) are unchanged, and CHANGELOG.md names the root cause and that it is fixed in the planning stage, not at one job.
5. [behavior] A new unit test in crates/athena-context asserts the planner brief and the planner and critic prompts contain the seam requirement; `cargo test -p athena-context` passes, and the test fails on the base commit.
6. [behavior] `cargo test -p athena-cli --test budgets` passes, including system_facts_budgets_match_computed and system_facts_drift_is_rejected.
7. [behavior] `cargo fmt --check`, `cargo clippy --locked` and `cargo test --locked` all pass, and the producer's report cites each command it ran with its result.

## Expected deliverables

- crates/athena-context/prompts/planner.md (modify)
- crates/athena-context/prompts/critic.md (modify)
- crates/athena-context/src/briefs.rs (modify)
- docs/system-facts.md (modify)
- CHANGELOG.md (modify)
- crates/athena-cli/tests/job_plan.rs (modify)
- crates/athena-context/src/lib.rs (modify)

## Out of scope

- Changing the three-round cap or the critic's verdict logic
- Adding fields to the Demonstration record or bumping its schema
- Redoing the landing, records or PR changes from the referenced jobs
- Changing CI workflows or governance files
- Raising the token budgets themselves

## User defaults that apply

- tested: Every module has passing tests; cargo test passes and clippy is clean. Here: New unit test plus full cargo test, clippy and fmt.
- outcome-check: The outcome is the check behind the work, not the activity done. For audience-facing quality, a fresh model reader that took no part in the work rates the live result as a newcomer, and a rating below 4 of 5 blocks completion. Prose-only: no loop code computes the rating yet. Here: Criteria check that prompts and brief carry the seam rule and that budgets tests pass.
- behavior-evidence: Acceptance criteria must check the requested behavior happening, not that types, files or wiring exist. A protection or denial test first shows the action succeeds without the protection, as the same user and in the same environment the real component runs as (root), then shows the action is refused. It fails, not skips or passes silently, if the environment cannot enforce the protection; a file mode that root bypasses is not proof. Worked examples are in docs/design/defaults-loop.md section 3.7. Here: Behavior criteria name the exact cargo commands to run.
- root-cause: Fix the root cause, not the symptom or the one instance that surfaced. The fault is fixed where it originates; when that lies outside the job's scope, name it and propose the fix rather than working around it. The change description names the root cause and why the change addresses it. The root-cause fix is the one focused change. Here: Fix in planning stage prompts and brief, not one job.
- small: Prefer one focused change; size is a design choice, not a wish list. Here: One focused prompt change plus its dependent size figures.
- changelog: (not in the defaults model) Here: BRIEF.md asks for tagged releases and a changelog, so the change records itself in CHANGELOG.md.

## Scope

The expected deliverables list is the one statement of scope. A changed file that is not on that list is the scope finding, judged against the list and not against the wording of a criterion.

## The change

Base 4725a813e598 → candidate 4c2319ee8c4d. In your working directory, `git diff HEAD~1` shows it. records/, the owner's private files, and history are absent by design; their absence is not a finding.

```text
 CHANGELOG.md                             |  9 +++++++++
 crates/athena-context/prompts/critic.md  |  1 +
 crates/athena-context/prompts/planner.md | 20 ++++++++++----------
 crates/athena-context/src/briefs.rs      | 24 +++++++++++++++++++++++-
 docs/system-facts.md                     |  4 ++--
 5 files changed, 45 insertions(+), 13 deletions(-)
```

(The diff was inlined for the verifier; it is elided in this record.)

## Checks Athena ran on this tree

- fmt: `cargo fmt --all --check` passed
- clippy: `cargo clippy --all-targets --locked -- -D warnings` passed
- test: `cargo test --locked` passed (1457 tests ran)
- criteria-left-to-athena-checks: `not a check` passed (the producer left these criteria to Athena's recorded checks (fmt, clippy, the full test suite); judge them yourself from the recorded check results: `cargo fmt --check`, `cargo clippy --locked` and `cargo test --locked` all pass, and the producer's report cites each command it ran with its result.)

## Commands Bash may run

- cargo build | check | test | clippy | fmt | run | metadata | tree
- git status | diff | log | show
- ls (paths inside your working directory)

## Verdict

End with the verdict as your structured output: one criteria entry per acceptance criterion, in order.
