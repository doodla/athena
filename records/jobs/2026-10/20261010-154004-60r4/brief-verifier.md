# Job 20261010-154004-60r4: verify "Record kill-switch stops as killed; mark bound-only cost provisional"

## Request (verbatim)

> Record a producer stopped by the kill switch as killed, and leave the cost score provisional when an execution cost is only a bound (m8, docs/design/reviews/athena-v0-acceptance-1.md).

## Reading (accepted by the user)

Fix m8: when the kill switch stops a producer before the driver's check, record the invocation as killed rather than error. Mark the cost_usd quantity score provisional when execution cost is only a bound. Audience is the maintainer and verifier reading records and scores.

Assumptions:
- Goal: fix minor m8 of the v0 acceptance review; audience is the maintainer and verifier reading invocation records and scores.
- Exact file paths in athena-score and athena-core are inferred; the producer locates them.
- No governance files need to change.

## Acceptance criteria

1. [behavior] A new driver test in crates/athena-agent/src/claude.rs shows a child ending with no result while the kill switch is on yields InvocationOutcome::Killed with a detail naming the kill switch, not Error. Run: cargo test -p athena-agent.
2. [behavior] A new scoring test in crates/athena-score builds a job whose execution cost is only a bound and shows the cost_usd quantity score is provisional; a job with exact costs is not provisional. Run: cargo test -p athena-score.
3. [behavior] An end-to-end test in crates/athena-cli/tests/lifecycle.rs shows a producer stopped by the kill switch closes with a killed invocation record and a provisional cost score, and the commit gate accepts the recomputed score. Run: cargo test -p athena-cli.
4. [behavior] Whole workspace passes cargo test and cargo clippy --all-targets with no warnings; cargo fmt --check is clean.
5. [content] A bound-only cost is no longer scored as a firm figure, and a provisional cost_usd score does not count in aggregates as a resolved value.
6. [content] Older score.json records without the new field still deserialize (serde default); existing history tests still pass.
7. [content] docs/records.md describes the provisional quantity field and kill-switch attribution, and CHANGELOG.md has an entry citing m8 of docs/design/reviews/athena-v0-acceptance-1.md.

## Expected deliverables

- crates/athena-agent/src/claude.rs (modify)
- crates/athena-score/src/lib.rs (modify)
- crates/athena-cli/tests/lifecycle.rs (modify)
- crates/athena-core/src/lib.rs (modify)
- docs/records.md (modify)
- CHANGELOG.md (modify)

## Out of scope

- Other minor findings of the v0 acceptance review
- Changing the kill switch polling interval

## User defaults that apply

- tested: Every module has passing tests; cargo test passes and clippy is clean. Here: New tests in agent, score, and cli crates; clippy and fmt criteria.
- behavior-evidence: Acceptance criteria must check the requested behavior happening, not that types, files or wiring exist. A protection or denial test first shows the action succeeds without the protection, as the same user and in the same environment the real component runs as (root), then shows the action is refused. It fails, not skips or passes silently, if the environment cannot enforce the protection; a file mode that root bypasses is not proof. Worked examples are in docs/design/defaults-loop.md section 3.7. Here: Criteria run tests that exercise killed and provisional outcomes.
- root-cause: Fix the root cause, not the symptom or the one instance that surfaced. The fault is fixed where it originates; when that lies outside the job's scope, name it and propose the fix rather than working around it. The change description names the root cause and why the change addresses it. The root-cause fix is the one focused change. Here: Fix the attribution in the driver and the scoring of bound costs, not one record.
- strangers: Code, commits, docs, and records are written for an outside reader. Here: Docs and changelog updated for outside readers.
- changelog: (not in the defaults model) Here: BRIEF.md asks for tagged releases and a changelog, so the change records itself in CHANGELOG.md.

## Scope

The expected deliverables list is the one statement of scope. A changed file that is not on that list is the scope finding, judged against the list and not against the wording of a criterion.

## The change

Base e0222881c889 → candidate 34074cf376ee. In your working directory, `git diff HEAD~1` shows it. records/, the owner's private files, and history are absent by design; their absence is not a finding.

```text
 CHANGELOG.md                         |   5 ++
 crates/athena-agent/src/claude.rs    |  37 ++++++++
 crates/athena-cli/tests/lifecycle.rs |  58 ++++++++++++
 crates/athena-core/src/scoring.rs    |   2 +-
 crates/athena-data/src/records.rs    |  26 ++++++
 crates/athena-score/src/lib.rs       | 168 ++++++++++++++++++++++++++++-------
 docs/records.md                      |  10 +++
 7 files changed, 275 insertions(+), 31 deletions(-)
```

(The diff was inlined for the verifier; it is elided in this record.)

## Checks Athena ran on this tree

- fmt: `cargo fmt --all --check` passed
- clippy: `cargo clippy --all-targets --locked -- -D warnings` passed
- test: `cargo test --locked` passed (852 tests ran)

## Commands Bash may run

- cargo build | check | test | clippy | fmt | run | metadata | tree
- git status | diff | log | show
- ls (paths inside your working directory)

## Verdict

End with the verdict as your structured output: one criteria entry per acceptance criterion, in order.
