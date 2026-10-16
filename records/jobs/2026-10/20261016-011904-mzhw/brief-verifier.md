# Job 20261016-011904-mzhw: verify "Send finished work to the verifier after an incomplete send-back report"

## Request (verbatim)

> Six jobs failed at the producer self-check (k2ej, 5qem, be8x, qb41, 5wwj, 582g). In 20261016-005953-582g the producer had done the work, but on the send-back pass its report covered only 3 of the 5 criteria, so the job failed over unreported ones. The send-back should make finished work reach the verifier, not fail it over an incomplete report.

## Reading (accepted by the user)

When the producer's report is still incomplete after the one send-back, run.rs fails the job ("still had problems when sent back"). Change it so that if the producer made changes, the work and last report go to the verifier, with the unreported criteria flagged, and only a no-change result fails. Audience: Athena operators whose jobs (e.g. 582g) failed over report gaps.

Assumptions:
- The fix is in the send-back loop in crates/athena-core/src/run.rs (~lines 714-723).
- Missing real full-test result after send-back also goes to the verifier, which can run cargo test itself; only 'no change' fails.
- Pinned-study jobs behave as before.
- I did not read the other five jobs' records; they are absent from my copy.

## Acceptance criteria

1. [behavior] A lifecycle test in which the producer, on the send-back pass, reports only some criteria but leaves real changes reaches the verifier and does not end in the 'still had problems when sent back' failure; run `cargo test`.
2. [behavior] A test where the producer makes no change after send-back still fails the job; run `cargo test`.
3. [content] The unreported or unmet criteria and any missing test result from the last report are kept in the attempt invocation's detail and passed to the verifier as context, so the verifier judges them rather than the self-check.
4. [content] The send-back prompt text in run.rs no longer says a still-missing criterion fails the job, and matches the new behavior.
5. [content] A CHANGELOG line and a decision entry record the change and why, citing job 582g.
6. [behavior] `cargo test`, `cargo clippy --all-targets` and `cargo fmt --check` all pass.

## Expected deliverables

- crates/athena-core/src/run.rs (modify)
- crates/athena-core/src/run_pin_tests.rs (modify)
- crates/athena-cli/tests/lifecycle.rs (modify)
- crates/athena-cli/tests/common/mod.rs (modify)
- CHANGELOG.md (modify)
- docs/decisions.md (modify)
- crates/athena-context/prompts/producer.md (modify)

## Out of scope

- Changing how many send-backs are allowed (PRODUCER_SEND_BACKS)
- Diagnosing the other five failed jobs individually
- Changing the verifier's own criteria checks

## User defaults that apply

- tested: Every module has passing tests; cargo test passes and clippy is clean. Here: Tests added for the new path; cargo test and clippy criterion.
- behavior-evidence: Acceptance criteria must check the requested behavior happening, not that types, files or wiring exist. A protection or denial test first shows the action succeeds without the protection, as the same user and in the same environment the real component runs as (root), then shows the action is refused. It fails, not skips or passes silently, if the environment cannot enforce the protection; a file mode that root bypasses is not proof. Worked examples are in docs/design/defaults-loop.md section 3.7. Here: Criteria run the lifecycle scenario rather than checking wiring.
- root-cause: Fix the root cause, not the symptom or the one instance that surfaced. The fault is fixed where it originates; when that lies outside the job's scope, name it and propose the fix rather than working around it. The change description names the root cause and why the change addresses it. The root-cause fix is the one focused change. Here: Change the send-back's give-up policy rather than special-casing job 582g.
- strangers: Code, commits, docs, and records are written for an outside reader. Here: Decision entry and CHANGELOG written for outside readers.
- small: Prefer one focused change; size is a design choice, not a wish list. Here: One focused change in the send-back path.
- changelog: (not in the defaults model) Here: BRIEF.md asks for tagged releases and a changelog, so the change records itself in CHANGELOG.md.

## Scope

The expected deliverables list is the one statement of scope. A changed file that is not on that list is the scope finding, judged against the list and not against the wording of a criterion.

## The change

Base 48582c050ba5 → candidate fbe23cb05369. In your working directory, `git diff HEAD~1` shows it.

```text
 CHANGELOG.md                              |  7 +++
 crates/athena-cli/tests/lifecycle.rs      | 98 ++++++++++++++++++++++++++-----
 crates/athena-context/prompts/producer.md |  2 +-
 crates/athena-core/src/run.rs             | 51 ++++++++++++----
 crates/athena-core/src/run_pin_tests.rs   |  7 ++-
 docs/decisions.md                         | 23 ++++++++
 6 files changed, 159 insertions(+), 29 deletions(-)
```

(The diff was inlined for the verifier; it is elided in this record.)

## Checks Athena ran on this tree

- fmt: `cargo fmt --all --check` passed
- clippy: `cargo clippy --all-targets --locked -- -D warnings` passed
- test: `cargo test --locked` passed (717 tests ran)

## Commands Bash may run

- cargo build | check | test | clippy | fmt | run | metadata | tree
- git status | diff | log | show
- ls (paths inside your working directory)

## Verdict

End with the verdict as your structured output: one criteria entry per acceptance criterion, in order.
