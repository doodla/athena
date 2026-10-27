# Job 20261027-045117-g5sr: verify "Add `athena hestia review --backfill` for repeated-cause backlog"

## Request (verbatim)

> Priority: urgent
> Run the error review over the backlog, not only on new introspections. athena status lists 32 repeated causes with no root-cause request (for example a stalled producer burning its full deadline, 9 times in 8 jobs; readings larger than one producer run, 7 in 7; criteria needing evidence the verifier cannot observe, 5 in 5), because the review queues only when a new introspection is recorded. Add `athena hestia review --backfill` (or the sweep's equivalent) that, once and idempotently: (1) skips causes whose stage is disabled in athena.toml (job planning is off, so planner- and critic-stage causes are listed as dormant, not queued); (2) for each remaining repeated cause, finds an existing open request that addresses it, including the steward requests the planner refuses for missing acceptance, and re-queues that request's text through the shared steward request builder, closing the old one with a reason; (3) otherwise queues a new root-cause request through the builder citing the introspection ids; (4) sets priority from occurrences and recency: the five highest-ranked urgent, the rest normal. Print what it queued, re-queued, skipped and why.
> Acceptance:
> - A test with fixture introspections and requests shows each of the four branches (dormant stage skipped, existing request re-queued and old one closed, new request queued, ranking sets urgent for the top five), and a second run changes nothing.
> - athena status shows each repeated cause's request after the backfill, and dormant causes separately.
> - fmt, clippy with -D warnings and the test suite pass; CHANGELOG.md has an entry.
> Files: crates/athena-core/src/hestia.rs, crates/athena-core/src/status.rs, crates/athena-cli/src/main.rs, CHANGELOG.md

## Reading (accepted by the user)

Add an idempotent backfill to the error review that sweeps repeated causes: skip dormant stages, re-queue an existing open request through the shared steward builder (closing the old one), or queue a new root-cause request. The five highest-ranked get urgent priority. Show the results in `athena status`. The audience is the Athena operator.

Assumptions:
- The stage-disabled check reads existing config in athena.toml; no config change is needed.
- The 'sweep equivalent' is satisfied by a CLI flag on the existing hestia review command.
- Idempotence comes from marking causes as covered by an open request after the first run.
- Matching an existing request to a cause uses the cause key or the cited introspection ids.

## Acceptance criteria

1. [behavior] Run `cargo test`: a fixture test in hestia.rs covers all four branches: a cause in a disabled stage is skipped as dormant, an existing open request is re-queued via the shared builder and the old one is closed with a reason, an uncovered cause gets a new request citing introspection ids, and the top five by occurrences and recency are urgent while the rest are normal.
2. [behavior] Run `cargo test`: a second backfill run over the same fixtures queues, re-queues and closes nothing.
3. [behavior] Run `cargo test`: status output lists each repeated cause with its request after the backfill and lists dormant causes in a separate section.
4. [behavior] Run `cargo run -- hestia review --backfill` help or a CLI test: the flag is accepted and prints what was queued, re-queued and skipped, with the reason for each.
5. [behavior] `cargo fmt --check`, `cargo clippy -- -D warnings` and `cargo test` all pass.
6. [content] CHANGELOG.md has an entry describing the backfill.
7. [content] Disabled stages are read from athena.toml, not hardcoded. Re-queued and new requests go through the existing shared steward request builder, with no duplicate construction path.

## Expected deliverables

- crates/athena-core/src/hestia.rs (modify)
- crates/athena-core/src/status.rs (modify)
- crates/athena-cli/src/main.rs (modify)
- CHANGELOG.md (modify)

## Out of scope

- Changing the planner's acceptance refusal behavior
- Enabling job planning in athena.toml
- Fixing the underlying root causes themselves

## User defaults that apply

- tested: Every module has passing tests; cargo test passes and clippy is clean. Here: Fixture tests added; cargo test, clippy and fmt are required to pass.
- behavior-evidence: Acceptance criteria must check the requested behavior happening, not that types, files or wiring exist. A protection or denial test first shows the action succeeds without the protection, as the same user and in the same environment the real component runs as (root), then shows the action is refused. It fails, not skips or passes silently, if the environment cannot enforce the protection; a file mode that root bypasses is not proof. Worked examples are in docs/design/defaults-loop.md section 3.7. Here: Criteria run the tests and check the four branches and the idempotent second run.
- outcome-check: The outcome is the check behind the work, not the activity done. For audience-facing quality, a fresh model reader that took no part in the work rates the live result as a newcomer, and a rating below 4 of 5 blocks completion. Prose-only: no loop code computes the rating yet. Here: The check is that status shows a request per repeated cause after the backfill.
- no-empty-success: An empty queue or finished list never counts as success on its own. Success needs a check of the outcome the list stood for. Prose-only: no loop code enforces it yet. Here: The tests need non-empty fixtures, and an empty backlog is not counted as success.
- small: Prefer one focused change; size is a design choice, not a wish list. Here: One focused change across the four listed files.
- rust: All code is Rust. Non-Rust files appear only where a platform requires them, and each is listed with its reason. Here: Implemented in Rust.
- changelog: (not in the defaults model) Here: BRIEF.md asks for tagged releases and a changelog, so the change records itself in CHANGELOG.md.

## Scope

The expected deliverables list is the one statement of scope. A changed file that is not on that list is the scope finding, judged against the list and not against the wording of a criterion.

## The change

Base efb6f0cd87b4 → candidate a64ba0e34dfd. In your working directory, `git diff HEAD~1` shows it. records/, the owner's private files, and history are absent by design; their absence is not a finding.

```text
 CHANGELOG.md                     |  10 +
 crates/athena-cli/src/display.rs |  55 +++
 crates/athena-cli/src/main.rs    |  42 ++
 crates/athena-core/src/hestia.rs | 946 ++++++++++++++++++++++++++++++++++++++-
 crates/athena-core/src/status.rs |  18 +-
 5 files changed, 1064 insertions(+), 7 deletions(-)
```

(The diff was inlined for the verifier; it is elided in this record.)

## Checks Athena ran on this tree

- fmt: `cargo fmt --all --check` passed
- clippy: `cargo clippy --all-targets --locked -- -D warnings` passed
- test: `cargo test --locked` passed (1414 tests ran)
- criteria-left-to-athena-checks: `not a check` passed (the producer left these criteria to Athena's recorded checks (fmt, clippy, the full test suite); judge them yourself from the recorded check results: `cargo fmt --check`, `cargo clippy -- -D warnings` and `cargo test` all pass.)

## Commands Bash may run

- cargo build | check | test | clippy | fmt | run | metadata | tree
- git status | diff | log | show
- ls (paths inside your working directory)

## Verdict

End with the verdict as your structured output: one criteria entry per acceptance criterion, in order.
