# Job 20261015-095955-qnxq: verify "Make drain approve in-limit proposals and work approved jobs"

## Request (verbatim)

> try again on 20261015-085652-5qem

## Reading (accepted by the user)

Drain currently lists audience implementation-rung proposals as "needs a person" and ignores the Proposed job that approve-work creates. Retry: drain approves open proposals whose rung the raising role may change itself, works jobs recorded as approved by a work decision, and its module docs say so. The audience is the operator running `athena drain`.

Assumptions:
- In-limit means the proposal's rung is at or below the raising role's limit; unknown roles and above-limit proposals stay waiting.
- Rejecting and approving above-limit proposals stay with people.
- A job already closed is not picked up again; an approved job read but still in state proposed is run once.
- The producer must run cargo fmt --check, clippy --all-targets and test and report evidence per criterion, including the new drain.rs text, before reporting done.

## Acceptance criteria

1. [behavior] A drain test gives an open proposal on the implementation rung from a role whose limit covers it. The proposal is approved, its job is read, run and landed, and it is absent from the report's waiting list. Run `cargo test -p athena-cli --test drain`.
2. [behavior] A drain test shows a proposal above its raising role's limit, or from an unknown role, is neither approved nor worked and still appears as waiting with the "needs a person" text. Run `cargo test -p athena-cli --test drain`.
3. [behavior] A drain test shows a job that a work decision records as approved and that is still in state proposed is picked up and run once by drain, and a closed job is not picked up again. Run `cargo test -p athena-cli --test drain`.
4. [content] The module doc in crates/athena-cli/src/drain.rs no longer contains "has no way to accept, approve or land". It states that in-limit proposals are approved and approved jobs are worked, and that rejecting and approving above-limit proposals stay with people.
5. [behavior] `cargo fmt --check`, `cargo clippy --all-targets` and `cargo test` pass for the workspace.

## Expected deliverables

- crates/athena-cli/src/drain.rs (modify)
- crates/athena-cli/tests/drain.rs (modify)
- crates/athena-core/src/work_decisions.rs (modify)
- crates/athena-core/src/steward.rs (modify)
- CHANGELOG.md (modify)
- crates/athena-cli/src/main.rs (modify)
- crates/athena-cli/src/display.rs (modify)
- docs/design/roles-and-ladder.md (modify)

## Out of scope

- Rejecting proposals from drain
- Approving proposals above a role's limit
- Changing the ladder or role limits themselves
- Changes to Athena's harness or pre-report gate

## User defaults that apply

- tested: Every module has passing tests; cargo test passes and clippy is clean. Here: New drain tests cover in-limit approval, above-limit waiting, and approved-job pickup; clippy and fmt must be clean.
- root-cause: Fix the root cause, not the symptom or the one instance that surfaced. The fault is fixed where it originates; when that lies outside the job's scope, name it and propose the fix rather than working around it. The change description names the root cause and why the change addresses it. The root-cause fix is the one focused change. Here: Fix both causes: the queue view ignores in-limit proposals and approved jobs.
- small: Prefer one focused change; size is a design choice, not a wish list. Here: One focused change in drain and the helpers it needs.
- strangers: Code, commits, docs, and records are written for an outside reader. Here: Update module docs, changelog and design doc so an outside reader sees the new behavior.
- changelog: (not in the defaults model) Here: BRIEF.md asks for tagged releases and a changelog, so the change records itself in CHANGELOG.md.

## Scope

The expected deliverables list is the one statement of scope. A changed file that is not on that list is the scope finding, judged against the list and not against the wording of a criterion.

## The change

Base 61d754bccd1e → candidate 8f4c29dc731b. In your working directory, `git diff HEAD~1` shows it.

```text
 CHANGELOG.md                             |   9 +
 README.md                                |   7 +-
 crates/athena-cli/src/display.rs         |   2 +-
 crates/athena-cli/src/drain.rs           | 168 ++++++++++++++++---
 crates/athena-cli/src/main.rs            |   3 +-
 crates/athena-cli/tests/drain.rs         | 278 +++++++++++++++++++++++++++++--
 crates/athena-core/src/steward.rs        |  29 +++-
 crates/athena-core/src/work_decisions.rs |  65 ++++++++
 docs/design/roles-and-ladder.md          |   6 +
 9 files changed, 531 insertions(+), 36 deletions(-)
```

(The diff was inlined for the verifier; it is elided in this record.)

## Checks Athena ran on this tree

- fmt: `cargo fmt --all --check` passed
- clippy: `cargo clippy --all-targets --locked -- -D warnings` passed
- test: `cargo test --locked` passed (678 tests ran)

## Commands Bash may run

- cargo build | check | test | clippy | fmt | run | metadata | tree
- git status | diff | log | show
- ls (paths inside your working directory)

## Verdict

End with the verdict as your structured output: one criteria entry per acceptance criterion, in order.
