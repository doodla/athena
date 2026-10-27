# Job 20261027-085401-0qgf: verify "Order approved steward proposals in the plan like other items"

## Request (verbatim)

> Priority: urgent
> athena drain works approved steward proposals before the plan: on 2026-10-27 its first drain approved proposal 20261018-000648-ne91 (a 9-day-old explainer-page fix), spent both of its job slots on it (failed twice) and never reached the urgent items at the head of athena plan. Approved proposals must enter the plan like any other item and be ordered by the same rules (dependency, then priority, then rung, then age), with a proposal's priority defaulting to normal unless its role or a priority record says otherwise.
> Acceptance:
> - A drain test with an approved in-limit proposal and an urgent queued request works the urgent request first.
> - athena plan lists approved proposals in their ordered place.
> - fmt, clippy with -D warnings and the test suite pass; CHANGELOG.md has an entry.
> Files: crates/athena-cli/src/drain.rs, crates/athena-core/src/plan.rs, CHANGELOG.md

## Reading (accepted by the user)

Drain currently approves in-limit steward proposals up front (approve_all) and works them before the plan, starving urgent queued requests. Make approved proposals plan entries ordered by dependency, priority, rung, age, with priority defaulting to normal unless the role or a priority record says otherwise; drain then follows the plan, and `athena plan` lists them in place. Audience: operators running `athena drain` and `athena plan`.

Assumptions:
- Approval still happens in drain, but approved proposals' jobs are merged into the plan and ordered rather than worked first.
- Priority for a proposal comes from its role or a priority record; absent either it is normal.
- No governance files need to change.

## Acceptance criteria

1. [behavior] A new drain test (run with cargo test -p athena-cli drain) with an approved in-limit proposal and an urgent queued request works the urgent request first, and fails on the old behavior.
2. [behavior] A test shows the plan (Plan::build / the plan command path) lists an approved proposal at its ordered place by dependency, priority, rung, then age, defaulting to normal priority, with role or priority-record overrides honored.
3. [content] Drain no longer works approved proposals ahead of the plan; approved proposals are ordered by the same rules as other items and job-limit/budget handling still applies.
4. [behavior] cargo fmt --check, cargo clippy --all-targets -- -D warnings and cargo test pass.
5. [content] CHANGELOG.md has an entry describing the fix for a reader outside the project.

## Expected deliverables

- crates/athena-cli/src/drain.rs (modify)
- crates/athena-core/src/plan.rs (modify)
- CHANGELOG.md (modify)

## Out of scope

- Fixing the explainer-page proposal 20261018-000648-ne91 itself
- Changing proposal approval limits or rung rules
- Changing the job limit semantics

## User defaults that apply

- tested: Every module has passing tests; cargo test passes and clippy is clean. Here: Tests for drain order and plan listing; fmt, clippy -D warnings, tests pass.
- behavior-evidence: Acceptance criteria must check the requested behavior happening, not that types, files or wiring exist. A protection or denial test first shows the action succeeds without the protection, as the same user and in the same environment the real component runs as (root), then shows the action is refused. It fails, not skips or passes silently, if the environment cannot enforce the protection; a file mode that root bypasses is not proof. Worked examples are in docs/design/defaults-loop.md section 3.7. Here: Criteria check order of work and plan output.
- root-cause: Fix the root cause, not the symptom or the one instance that surfaced. The fault is fixed where it originates; when that lies outside the job's scope, name it and propose the fix rather than working around it. The change description names the root cause and why the change addresses it. The root-cause fix is the one focused change. Here: Fix ordering by putting proposals into the plan, not special-casing the one proposal.
- verified: Done means independently verified, and user-facing behavior is verified by exercising it. Here: Verifier runs cargo test, clippy, fmt.
- outcome-check: The outcome is the check behind the work, not the activity done. For audience-facing quality, a fresh model reader that took no part in the work rates the live result as a newcomer, and a rating below 4 of 5 blocks completion. Prose-only: no loop code computes the rating yet. Here: Urgent request is worked first.
- changelog: (not in the defaults model) Here: BRIEF.md asks for tagged releases and a changelog, so the change records itself in CHANGELOG.md.

## Scope

The expected deliverables list is the one statement of scope. A changed file that is not on that list is the scope finding, judged against the list and not against the wording of a criterion.

## The change

Base e4687ee0d5c3 → candidate 708a21044f72. In your working directory, `git diff HEAD~1` shows it. records/, the owner's private files, and history are absent by design; their absence is not a finding.

```text
 CHANGELOG.md                     |  14 ++++
 crates/athena-cli/src/drain.rs   |  80 +++++++++++++++----
 crates/athena-cli/tests/drain.rs | 165 +++++++++++++++++++++++++++++++++++++++
 crates/athena-core/src/plan.rs   | 104 ++++++++++++++++++++++++
 4 files changed, 349 insertions(+), 14 deletions(-)
```

(The diff was inlined for the verifier; it is elided in this record.)

## Checks Athena ran on this tree

- fmt: `cargo fmt --all --check` passed
- clippy: `cargo clippy --all-targets --locked -- -D warnings` passed
- test: `cargo test --locked` passed (1475 tests ran)
- criteria-left-to-athena-checks: `not a check` passed (the producer left these criteria to Athena's recorded checks (fmt, clippy, the full test suite); judge them yourself from the recorded check results: cargo fmt --check, cargo clippy --all-targets -- -D warnings and cargo test pass.)

## Commands Bash may run

- cargo build | check | test | clippy | fmt | run | metadata | tree
- git status | diff | log | show
- ls (paths inside your working directory)

## Verdict

End with the verdict as your structured output: one criteria entry per acceptance criterion, in order.
