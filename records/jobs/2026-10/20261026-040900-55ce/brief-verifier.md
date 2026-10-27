# Job 20261026-040900-55ce: verify "Flag unchanged planned files before verification starts"

## Request (verbatim)

> Before the producer hands in a candidate, compare its diff with the plan's planned files and expected deliverables. If any are left unchanged, the producer must either change them or say why in its report, before verification starts. In both jobs (branch operator/next5) the verifier found a major finding: the listed test files under crates/athena-cli/tests were not changed. In y179 the verifier also says common/mod.rs and run.rs were not changed. Add a test that a candidate missing a planned file is flagged. Note: the missed time target in these same jobs is already queued as performance-target-criterion-set-without-measured-feasibility. This item is only about the missing planned files.

## Reading (accepted by the user)

Add a pre-verification check in the producer loop (crates/athena-core/src/run/mod.rs): compare the candidate's change list with the accepted job plan's files (create/modify/delete) and the reading's deliverables. Each one left unchanged is sent back with the criteria send-back, or the producer's report must say why. If it is still unexplained, the gap is flagged for the verifier. Include a test that a candidate missing a planned file is flagged. Audience: Athena maintainers and the verifier, who found these gaps late.

Assumptions:
- The mechanism extends the existing self-check and send-back (unmet_criteria, send_back_text, PRODUCER_SEND_BACKS): it runs on the producer's report and the snapshot change list before the candidate is built, and shares the one send-back rather than adding a second.
- Planned files come from the accepted JobPlan.files, and expected deliverables from the accepted reading's deliverables. A planned file counts as changed when its path is in the snapshot's change list. Deleted planned files count when the action is delete.
- Producer output may need an optional field (for example unchanged_planned_files with a reason per path) so a justification can be machine-checked; if the schema already has a suitable field, reuse it.
- The check is a flag and send-back, not a hard failure: after the send-back the gap goes to the verifier, matching how unmet criteria are handled today.
- Jobs without planning have only the reading's deliverables to compare. The missed time target (performance-target-criterion-set-without-measured-feasibility) is out of scope.

## Acceptance criteria

1. [behavior] A new unit test builds a candidate change list that omits one planned file (for example a crates/athena-cli/tests path in the plan's files) and asserts the check returns that path as flagged; run with cargo test -p athena-core.
2. [behavior] A new test shows a candidate that changes every planned file, or whose report gives a reason for an unchanged one, is not flagged; run with cargo test -p athena-core.
3. [behavior] A test of the producer loop shows an unexplained missing planned file triggers the send-back text naming the path, and that after the send-back the gap is recorded for the verifier rather than silently passing; run with cargo test -p athena-core.
4. [content] The check runs in the producer step before verification starts, and the send-back brief and the producer prompt state that planned files left unchanged must be changed or explained in the report.
5. [content] The missed time target is not touched; the change only concerns unchanged planned files and deliverables.
6. [behavior] cargo test, cargo clippy and cargo fmt --check pass for the workspace.

## Expected deliverables

- crates/athena-core/src/run/mod.rs (modify)
- crates/athena-core/src/run_pin_tests.rs (modify)
- crates/athena-context/src/briefs.rs (modify)
- crates/athena-context/src/outputs.rs (modify)
- CHANGELOG.md (modify)
- docs/design/job-planning.md (modify)

## Out of scope

- The missed time target (performance-target-criterion-set-without-measured-feasibility).
- Changing the verifier's own judgment or findings logic.
- Editing athena.toml, defaults.toml or workflow files.

## User defaults that apply

- tested: Every module has passing tests; cargo test passes and clippy is clean. Here: New tests for the check; cargo test and clippy must pass.
- behavior-evidence: Acceptance criteria must check the requested behavior happening, not that types, files or wiring exist. A protection or denial test first shows the action succeeds without the protection, as the same user and in the same environment the real component runs as (root), then shows the action is refused. It fails, not skips or passes silently, if the environment cannot enforce the protection; a file mode that root bypasses is not proof. Worked examples are in docs/design/defaults-loop.md section 3.7. Here: Criteria check a candidate with a missing planned file being flagged, not that code exists.
- root-cause: Fix the root cause, not the symptom or the one instance that surfaced. The fault is fixed where it originates; when that lies outside the job's scope, name it and propose the fix rather than working around it. The change description names the root cause and why the change addresses it. The root-cause fix is the one focused change. Here: Fixes the producer hand-in gap, not just the two named jobs.
- small: Prefer one focused change; size is a design choice, not a wish list. Here: One focused change to the producer self-check.
- verified: Done means independently verified, and user-facing behavior is verified by exercising it. Here: Behavior criteria name cargo test commands the verifier can run.
- outcome-check: The outcome is the check behind the work, not the activity done. For audience-facing quality, a fresh model reader that took no part in the work rates the live result as a newcomer, and a rating below 4 of 5 blocks completion. Prose-only: no loop code computes the rating yet. Here: The check is the flag on a missing planned file, not the activity of comparing.
- strangers: Code, commits, docs, and records are written for an outside reader. Here: Doc comments and changelog written for an outside reader.
- changelog: (not in the defaults model) Here: BRIEF.md asks for tagged releases and a changelog, so the change records itself in CHANGELOG.md.

## Scope

The expected deliverables list is the one statement of scope. A changed file that is not on that list is the scope finding, judged against the list and not against the wording of a criterion.

## The change

Base f84698df2987 → candidate be8ab043b859. In your working directory, `git diff HEAD~1` shows it. records/, the owner's private files, and history are absent by design; their absence is not a finding.

```text
 CHANGELOG.md                            |  10 ++
 crates/athena-agent/src/fake.rs         |  12 +-
 crates/athena-cli/tests/common/mod.rs   |  30 ++++
 crates/athena-context/src/briefs.rs     |  14 ++
 crates/athena-context/src/outputs.rs    |  16 ++
 crates/athena-core/src/run/mod.rs       | 133 ++++++++++++++---
 crates/athena-core/src/run_pin_tests.rs | 253 ++++++++++++++++++++++++++++++++
 docs/design/job-planning.md             |  10 ++
 8 files changed, 457 insertions(+), 21 deletions(-)
```

(The diff was inlined for the verifier; it is elided in this record.)

## Checks Athena ran on this tree

- fmt: `cargo fmt --all --check` passed
- clippy: `cargo clippy --all-targets --locked -- -D warnings` passed
- test: `cargo test --locked` passed (1441 tests ran)
- criteria-left-to-athena-checks: `not a check` passed (the producer left these criteria to Athena's recorded checks (fmt, clippy, the full test suite); judge them yourself from the recorded check results: cargo test, cargo clippy and cargo fmt --check pass for the workspace.)

## Commands Bash may run

- cargo build | check | test | clippy | fmt | run | metadata | tree
- git status | diff | log | show
- ls (paths inside your working directory)

## Verdict

End with the verdict as your structured output: one criteria entry per acceptance criterion, in order.
