# Job 20261105-163159-vsfy: verify "Find and remove the cause of the longer verifier stage"

## Request (verbatim)

> Priority: high
> 
> Throughput slowdown: verifier grew from 3.1 to 3.5 min per landed job
> 
> Athena's throughput review found that the median minutes per landed job rose from 616.3 to 1032.3, more than 25%, since the previous throughput review. Find why the stage that grew most now takes longer and remove the cause.
> 
> Stage that grew most: verifier, a median of 3.5 minutes per landed job now, 3.1 before.
> Stages whose median minutes grew, in minutes per landed job before to now: verifier 3.1 to 3.5.
> 
> The medians are over the newest landed jobs that Athena measured. The status section of athena status prints the same stage medians.
> 
> Acceptance:
> - The cause of the longer verifier stage is found and stated in the change or in a note under docs/design/
> - The change shortens that stage, and the next throughput review shows the median minutes per landed job falling
> 
> Files: crates/, docs/design/

## Reading (accepted by the user)

The verifier stage median rose from 3.1 to 3.5 min per landed job. The producer finds the cause in the verifier path, fixes it so the stage gets shorter, and states the cause in a docs/design note. The audience is the maintainer reading the next throughput review.

Assumptions:
- Code survey: read crates/athena-core/src/status.rs. It defines the stage medians (the verifier column is role_minutes over Role::Verifier, around lines 651-931), and `athena status` prints them. The producer must also survey the verifier run path in crates/athena-core/src/run/ and the checks the verifier runs, to find where time is spent. No record or data file is added under crates/, so check_record_file, AnyRecord and the round-trip test are not affected.
- The verifier time is measured directly by role_minutes on the verifier invocations, not through a stand-in.
- Cause is not yet known. The producer must measure it from the code (for example repeated or cold builds, redundant checks, a larger prompt or diff context for the verifier) and not guess.
- The change makes no new refusal rule, so no refused-reading percentage applies.
- Out of scope: rewording the other stages, and changing records/ or the throughput review thresholds.

## Acceptance criteria

1. [content] A note under docs/design/ (or the commit message) states the cause of the longer verifier stage and cites the code path or measurement that shows it, and the verifier can read it in the diff.
2. [content] The diff removes the stated cause in crates/ code in the verifier path, for example by removing redundant work, and does not only change how the median is reported.
3. [behavior] Run `cargo test` from the repository root: it passes, including a new or updated test that exercises the changed verifier behavior, and it fails if the cause is reintroduced.
4. [behavior] Run `cargo clippy` and `cargo fmt --check`: both are clean.
5. [content] The change is a single focused fix with no unrelated edits, and the note says how the next throughput review should show verifier minutes per landed job falling below 3.5.

## Expected deliverables

- crates/athena-core/src/run/mod.rs (modify)
- crates/athena-core/src/status.rs (modify)
- docs/design/verifier-stage-slowdown.md (create)
- crates/athena-core/src/run/tests.rs (modify)
- CHANGELOG.md (modify)

## Out of scope

- Changes to the other stages' timing
- Changes to records/ or to the throughput review thresholds

## User defaults that apply

- root-cause: Fix the root cause, not the symptom or the one instance that surfaced. The fault is fixed where it originates; when that lies outside the job's scope, name it and propose the fix rather than working around it. The change description names the root cause and why the change addresses it. The root-cause fix is the one focused change. Here: The cause must be found and removed, not hidden.
- tested: Every module has passing tests; cargo test passes and clippy is clean. Here: cargo test and clippy criteria.
- behavior-evidence: Acceptance criteria must check the requested behavior happening, not that types, files or wiring exist. A protection or denial test first shows the action succeeds without the protection, as the same user and in the same environment the real component runs as (root), then shows the action is refused. It fails, not skips or passes silently, if the environment cannot enforce the protection; a file mode that root bypasses is not proof. Worked examples are in docs/design/defaults-loop.md section 3.7. Here: The criteria check the fix and a test, not just that files exist.
- outcome-check: The outcome is the check behind the work, not the activity done. For audience-facing quality, a fresh model reader that took no part in the work rates the live result as a newcomer, and a rating below 4 of 5 blocks completion. Prose-only: no loop code computes the rating yet. Here: The goal is that verifier minutes fall.
- strangers: Code, commits, docs, and records are written for an outside reader. Here: The design note is written for an outside reader.
- evidence: Claims cite their sources, and numbers are measured or labeled as estimates. Here: The note cites measured sources.
- changelog: (not in the defaults model) Here: BRIEF.md asks for tagged releases and a changelog, so the change records itself in CHANGELOG.md.

## Scope

The expected deliverables list is the one statement of scope. A changed file that is not on that list is the scope finding, judged against the list and not against the wording of a criterion.

## Soundness

Would a careful senior engineer reject this approach as a crude heuristic, a special case that should be general, a guess where the code or data could be read, or a rule with obvious false positives (for example, one that matches common words)? Name every such problem, each with the simpler or more correct alternative. Judge the approach only, not style.

A blocker or major soundness finding fails the verdict like an unmet criterion, even when every criterion is met. Minor and nit findings do not.

## Replay of accepted readings

Athena ran the newest accepted readings through the base and the candidate reading rules: 0 of 50 are newly refused by the candidate and were accepted by the base. This is within the allowed share, so the job is not held.

## The change

Base 8408650bfd90 → candidate 381f1b0befce. In your working directory, `git diff HEAD~1` shows it. records/, the owner's private files, and history are absent by design; their absence is not a finding.

```text
 CHANGELOG.md                               |   9 +++
 crates/athena-core/src/run/checks.rs       | 113 +++++++++++++++++++++++++++++
 crates/athena-core/src/run/tests.rs        |  93 ++++++++++++++++++++----
 crates/athena-core/src/run/verification.rs |   6 ++
 docs/design/verifier-stage-slowdown.md     |  61 ++++++++++++++++
 5 files changed, 268 insertions(+), 14 deletions(-)
```

(The diff was inlined for the verifier; it is elided in this record.)

## Evidence snapshot

Athena copied the real records when this job was dispatched into a read-only directory outside your working copy:

- path: <state>/jobs/20261105-163159-vsfy/evidence

It holds `records/` (every file under records/ at the base commit), `landings/` (the landing records) and `history.json` (the base commit and the newest 200 first-parent commits of main: hash, commit time, subject, trailers). Every check of real records, such as counting jobs, reading an earlier job's records or comparing a claim with history, uses this snapshot and not the working copy. Read it with the Read, Grep and Glob tools, or run `cargo run -p athena-cli -- status --snapshot <state>/jobs/20261105-163159-vsfy/evidence`, which prints the job counts, the history summary and the throughput section from it. It is read-only: a write to it fails. The working copy still has no records/.

## Checks Athena ran on this tree

- fmt: `cargo fmt --all --check` passed (result reused from the CI run https://github.com/doodla/athena/actions/runs/42105903361 for 381f1b0befce9fb670a4d74d3427f878ad99abe6)
- clippy: `cargo clippy --all-targets --locked -- -D warnings` passed (result reused from the CI run https://github.com/doodla/athena/actions/runs/42105903361 for 381f1b0befce9fb670a4d74d3427f878ad99abe6)
- test: `cargo test --locked` passed (result reused from the CI run https://github.com/doodla/athena/actions/runs/42105903361 for 381f1b0befce9fb670a4d74d3427f878ad99abe6)
- ci-result: `not a check` passed (Check results for 381f1b0befce9fb670a4d74d3427f878ad99abe6 come from the CI run https://github.com/doodla/athena/actions/runs/42105903361 (649s): fmt success, clippy success, test success.)
- criteria-left-to-athena-checks: `not a check` passed (the producer left these criteria to Athena's recorded checks (fmt, clippy, the full test suite); judge them yourself from the recorded check results: Run `cargo clippy` and `cargo fmt --check`: both are clean.)

## Commands Bash may run

- cargo build | check | test | clippy | fmt | run | metadata | tree
- git status | diff | log | show
- ls (paths inside your working directory)

## Verdict

End with the verdict as your structured output: one criteria entry per acceptance criterion, in order.
