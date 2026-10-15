# Job 20261015-070754-zy4a: verify "Add drain command that lands via PR and rates the changed page"

## Request (verbatim)

> try again on 20261015-064153-wekz

## Reading (accepted by the user)

Add an `athena drain` command that works the steward/audience queue itself within a user-set budget: run each item, land it through the `athena land --pr` path, then rate the live page. This retry fixes the rejected attempt's gap: a rating counts as "target met" only if taken after a landing. Audience is the repository owner, who now runs, lands and rates by hand.

Assumptions:
- The base has no drain command; the earlier attempt's changes were discarded, so this builds it whole, including the corrected land step.
- Landing uses the same path as `athena land --pr` (Athena::land_pr / gh::land_pr), behind a trait or injected step so tests can stub it.
- Stop reasons: target met, budget spent, repeated failure (same item failing twice), queue empty (not success). Items needing a person (above-ladder proposals) are listed as waiting and never accepted or landed.
- 'Target met' needs a rating taken after at least one landing in this run. If nothing landed, the output says the rating is of an unchanged page and does not report target met.
- A budget check guards every model call, including the retry re-reading, not just job starts: spend plus the next call's cap must fit.
- No new config key; the budget, rating threshold and job limit are CLI flags, recorded in docs/decisions.md.
- tests/drain.rs is expected as a new file this time.

## Acceptance criteria

1. [behavior] `cargo test` passes, including fake-agent drain tests with a stubbed queue and stubbed land step that show: (1) each worked item is landed through the land step, (2) the rating is taken only after landing, (3) a pass that landed nothing cannot report 'target met' from a stale rating and the output says the page was unchanged.
2. [behavior] Tests cover: two items landed then rating 4+ gives 'target met'; budget too small for the next call gives 'budget spent' with remaining items still queued; empty queue stops at once with no spend and says empty, not success; same item failing twice gives 'repeated failure' naming the item; a landing failure does not lead to a rating being taken as success.
3. [behavior] A unit test pins that no job or model call (including the retry re-reading) starts when recorded spend plus its cap would exceed the budget, including when an earlier job's cost exceeded its estimate.
4. [behavior] A test shows an above-ladder item needing a person is listed as waiting and left untouched, never accepted, approved or landed by the loop.
5. [behavior] `cargo clippy --all-targets` and `cargo fmt --check` are clean; `cargo run -p athena-cli -- drain --help` documents the budget, rating-threshold and job-limit flags and lists the stop reasons.
6. [content] The final output (text and --json) names the stop reason, jobs run with outcomes, costs and landing status, total spent, the latest rating against the target and whether it was taken after a landing, and the items still queued or waiting for a person.
7. [content] README.md 'What it does not decide', CHANGELOG.md and docs/decisions.md describe the command, that it lands through the PR path before rating, budget semantics, stop reasons, and the no-config-key decision; no text claims the owner must land by hand.

## Expected deliverables

- crates/athena-cli/src/drain.rs (create)
- crates/athena-cli/tests/drain.rs (create)
- crates/athena-cli/src/main.rs (modify)
- crates/athena-cli/src/lib.rs (modify)
- crates/athena-cli/src/display.rs (modify)
- crates/athena-core/src/steward.rs (modify)
- crates/athena-core/src/lib.rs (modify)
- README.md (modify)
- CHANGELOG.md (modify)
- docs/decisions.md (modify)

## Out of scope

- Accepting, approving or landing above-ladder proposals that need a person.
- Adding a config key for the budget or threshold.
- Changing how the audience rating itself is produced.
- Changes to athena.toml, defaults.toml or workflows.

## User defaults that apply

- tested: Every module has passing tests; cargo test passes and clippy is clean. Here: Fake-agent and stubbed-land tests for each stop reason and the land-before-rate ordering; clippy and fmt clean.
- verified: Done means independently verified, and user-facing behavior is verified by exercising it. Here: Criteria are checkable with cargo test, clippy, fmt and `drain --help`.
- root-cause: Fix the root cause, not the symptom or the one instance that surfaced. The fault is fixed where it originates; when that lies outside the job's scope, name it and propose the fix rather than working around it. The change description names the root cause and why the change addresses it. The root-cause fix is the one focused change. Here: Fixes the missing land step instead of only documenting it, and guards the retry call with the budget.
- strangers: Code, commits, docs, and records are written for an outside reader. Here: README, CHANGELOG and decisions entries written for an outside reader.
- mechanism: Rules that matter are enforced by mechanism, or explicitly marked prose-only. Here: The 'rated after landing' rule is enforced in code and pinned by tests, not by prose.
- small: Prefer one focused change; size is a design choice, not a wish list. Here: One command in a new module with minimal edits elsewhere.
- changelog: (not in the defaults model) Here: BRIEF.md asks for tagged releases and a changelog, so the change records itself in CHANGELOG.md.

## Scope

The expected deliverables list is the one statement of scope. A changed file that is not on that list is the scope finding, judged against the list and not against the wording of a criterion.

## The change

Base 321253ecab23 → candidate 1fe34f86fef3. In your working directory, `git diff HEAD~1` shows it.

```text
 CHANGELOG.md                      |  12 +
 README.md                         |  24 ++
 crates/athena-cli/src/display.rs  |  85 ++++
 crates/athena-cli/src/drain.rs    | 850 ++++++++++++++++++++++++++++++++++++++
 crates/athena-cli/src/lib.rs      |   1 +
 crates/athena-cli/src/main.rs     |  70 +++-
 crates/athena-cli/tests/drain.rs  | 569 +++++++++++++++++++++++++
 crates/athena-core/src/lib.rs     |   3 +-
 crates/athena-core/src/steward.rs |  75 +++-
 docs/decisions.md                 |  42 ++
 10 files changed, 1726 insertions(+), 5 deletions(-)
```

(The diff was inlined for the verifier; it is elided in this record.)

## Checks Athena ran on this tree

- fmt: `cargo fmt --all --check` passed
- clippy: `cargo clippy --all-targets --locked -- -D warnings` passed
- test: `cargo test --locked` passed (659 tests ran)

## Commands Bash may run

- cargo build | check | test | clippy | fmt | run | metadata | tree
- git status | diff | log | show
- ls (paths inside your working directory)

## Verdict

End with the verdict as your structured output: one criteria entry per acceptance criterion, in order.
