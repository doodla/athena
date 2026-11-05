# Job 20261105-093606-199q: verify "Find and remove the cause of the longer checks stage"

## Request (verbatim)

> Priority: high
> 
> Throughput slowdown: checks grew from 0.2 to 0.6 min per landed job
> 
> Athena's throughput review found that the median minutes per landed job rose from 464.5 to 616.3, more than 25%, since the previous throughput review. Find why the stage that grew most now takes longer and remove the cause.
> 
> Stage that grew most: checks, a median of 0.6 minutes per landed job now, 0.2 before.
> Stages whose median minutes grew, in minutes per landed job before to now: checks 0.2 to 0.6.
> 
> The medians are over the newest landed jobs that Athena measured. The status section of athena status prints the same stage medians.
> 
> Acceptance:
> - The cause of the longer checks stage is found and stated in the change or in a note under docs/design/
> - The change shortens that stage, and the next throughput review shows the median minutes per landed job falling
> 
> Files: crates/, docs/design/

## Reading (accepted by the user)

Athena's checks stage median rose from 0.2 to 0.6 min per landed job. Find the cause in the checks path (routing, CI result reuse, baseline lookup, executor), state it in a docs/design note, and fix it so the stage gets shorter. The audience is the maintainer reading the next throughput review.

Assumptions:
- Code survey: read crates/athena-core/src/run/checks.rs (SnapExec runs one routed check after a one-time disk gate; baseline lookup uses RefuseExec and WaitsWhileRunning, with a bounded CI read loop in crates/athena-core/src/ci_result.rs). Related, not yet read: crates/athena-core/src/routing.rs, crates/athena-core/src/status.rs (stage medians). No record or data file is added under crates/, so check_record_file, AnyRecord and the round-trip test are not affected.
- The exact cause is not known from the request. The producer must find it by reading the checks path (what a check runs, CI result reuse versus local runs, baseline wait bound, snapshot restore after each check) and recent changes to it, and must measure it directly, not through a stand-in.
- The change is a single focused fix plus a short note in docs/design/ stating the cause with evidence; no refusal rule is added or tightened.
- The next throughput review cannot be run in the job; the verifier checks that the change removes work or waiting from the checks path, not the future median.
- Governance files (athena.toml, defaults.toml, workflows) are not expected to change.

## Acceptance criteria

1. [content] A note under docs/design/ (or the change's commit text/doc comment) states the cause of the longer checks stage, citing the specific crates/ code path and the evidence (measured or labeled estimate).
2. [content] The diff to crates/ removes or bounds the identified cause (extra check run, wait, repeated work or snapshot restore) in the checks path, rather than a symptom such as raising a limit.
3. [behavior] A new or updated test in crates/ exercises the fixed behavior, failing on the old behavior; run `cargo test` and it passes.
4. [behavior] `cargo clippy --all-targets` is clean and `cargo fmt --check` passes.
5. [content] Existing checks behavior is preserved: the same routed checks still run and failures still hold or reject the job; existing tests in crates/athena-core/src/run/tests.rs still pass.

## Expected deliverables

- crates/athena-core/src/run/checks.rs (modify)
- crates/athena-core/src/routing.rs (modify)
- crates/athena-core/src/ci_result.rs (modify)
- crates/athena-core/src/run/tests.rs (modify)
- docs/design/checks-stage-slowdown.md (create)
- crates/athena-core/src/run/mod.rs (modify)
- CHANGELOG.md (modify)

## Out of scope

- Changing other stages (planning, producer, verification) that did not grow
- Changing athena.toml, defaults.toml or CI workflow files
- Changing how throughput medians are computed or reported

## User defaults that apply

- rust: All code is Rust. Non-Rust files appear only where a platform requires them, and each is listed with its reason. Here: Fix is Rust code in crates/.
- tested: Every module has passing tests; cargo test passes and clippy is clean. Here: Criterion requires cargo test and clippy to pass with a test for the fix.
- root-cause: Fix the root cause, not the symptom or the one instance that surfaced. The fault is fixed where it originates; when that lies outside the job's scope, name it and propose the fix rather than working around it. The change description names the root cause and why the change addresses it. The root-cause fix is the one focused change. Here: The cause must be found and removed, not worked around.
- behavior-evidence: Acceptance criteria must check the requested behavior happening, not that types, files or wiring exist. A protection or denial test first shows the action succeeds without the protection, as the same user and in the same environment the real component runs as (root), then shows the action is refused. It fails, not skips or passes silently, if the environment cannot enforce the protection; a file mode that root bypasses is not proof. Worked examples are in docs/design/defaults-loop.md section 3.7. Here: Test criterion checks the fixed behavior by running cargo test.
- outcome-check: The outcome is the check behind the work, not the activity done. For audience-facing quality, a fresh model reader that took no part in the work rates the live result as a newcomer, and a rating below 4 of 5 blocks completion. Prose-only: no loop code computes the rating yet. Here: The goal is a shorter checks stage, so the cause is stated with evidence.
- small: Prefer one focused change; size is a design choice, not a wish list. Here: One focused fix plus one note.
- strangers: Code, commits, docs, and records are written for an outside reader. Here: The design note is written for an outside reader.
- changelog: (not in the defaults model) Here: BRIEF.md asks for tagged releases and a changelog, so the change records itself in CHANGELOG.md.

## Scope

The expected deliverables list is the one statement of scope. A changed file that is not on that list is the scope finding, judged against the list and not against the wording of a criterion.

## Soundness

Would a careful senior engineer reject this approach as a crude heuristic, a special case that should be general, a guess where the code or data could be read, or a rule with obvious false positives (for example, one that matches common words)? Name every such problem, each with the simpler or more correct alternative. Judge the approach only, not style.

A blocker or major soundness finding fails the verdict like an unmet criterion, even when every criterion is met. Minor and nit findings do not.

## Replay of accepted readings

Athena ran the newest accepted readings through the base and the candidate reading rules: 0 of 50 are newly refused by the candidate and were accepted by the base. This is within the allowed share, so the job is not held.

## The change

Base 03a4910eafe4 → candidate 999e5b0deb69. In your working directory, `git diff HEAD~1` shows it. records/, the owner's private files, and history are absent by design; their absence is not a finding.

```text
 CHANGELOG.md                         |  5 ++
 crates/athena-core/src/run/checks.rs | 44 ++++++++++++++----
 crates/athena-core/src/run/tests.rs  | 79 ++++++++++++++++++++++++++++++++
 crates/athena-core/src/verify.rs     | 89 ++++++++++++++++++++++++++++++++++++
 docs/design/checks-stage-slowdown.md | 61 ++++++++++++++++++++++++
 5 files changed, 268 insertions(+), 10 deletions(-)
```

(The diff was inlined for the verifier; it is elided in this record.)

## Evidence snapshot

Athena copied the real records when this job was dispatched into a read-only directory outside your working copy:

- path: <state>/jobs/20261105-093606-199q/evidence

It holds `records/` (every file under records/ at the base commit), `landings/` (the landing records) and `history.json` (the base commit and the newest 200 first-parent commits of main: hash, commit time, subject, trailers). Every check of real records, such as counting jobs, reading an earlier job's records or comparing a claim with history, uses this snapshot and not the working copy. Read it with the Read, Grep and Glob tools, or run `cargo run -p athena-cli -- status --snapshot <state>/jobs/20261105-093606-199q/evidence`, which prints the job counts, the history summary and the throughput section from it. It is read-only: a write to it fails. The working copy still has no records/.

## Checks Athena ran on this tree

- fmt: `cargo fmt --all --check` passed (result reused from the CI run https://github.com/doodla/athena/actions/runs/42006059478 for 999e5b0deb69ca7fe9cffc9f223a3ceb7c47b12f)
- clippy: `cargo clippy --all-targets --locked -- -D warnings` passed (result reused from the CI run https://github.com/doodla/athena/actions/runs/42006059478 for 999e5b0deb69ca7fe9cffc9f223a3ceb7c47b12f)
- test: `cargo test --locked` passed (result reused from the CI run https://github.com/doodla/athena/actions/runs/42006059478 for 999e5b0deb69ca7fe9cffc9f223a3ceb7c47b12f)
- ci-result: `not a check` passed (Check results for 999e5b0deb69ca7fe9cffc9f223a3ceb7c47b12f come from the CI run https://github.com/doodla/athena/actions/runs/42006059478 (643s): fmt success, clippy success, test success.)
- criteria-left-to-athena-checks: `not a check` passed (the producer left these criteria to Athena's recorded checks (fmt, clippy, the full test suite); judge them yourself from the recorded check results: `cargo clippy --all-targets` is clean and `cargo fmt --check` passes.)

## Commands Bash may run

- cargo build | check | test | clippy | fmt | run | metadata | tree
- git status | diff | log | show
- ls (paths inside your working directory)

## Verdict

End with the verdict as your structured output: one criteria entry per acceptance criterion, in order.
