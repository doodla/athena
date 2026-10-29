# Job 20261029-073414-yg89: verify "Design an asynchronous, pipelined drain in docs/design/async-drain.md"

## Request (verbatim)

> Priority: urgent
> Design an asynchronous, pipelined drain (Sri, 2026-10-29: 'Heard of async?'). Today drain runs each job start to finish before the next begins, so every wait blocks everything: measured over 2026-10-27 to 29, a landed job takes a median 42 minutes end to end (producer 15, Athena's checks 13 to 15, landing with the early CI wait, push and self-rebuild about 6 to 8, verifier 2.3, reading and planning up to 5), and drain landed 35 of 61 jobs in two days. Write docs/design/async-drain.md proposing: (1) reading and hard-job planning run ahead for the next items while the current job builds; (2) the producer of the next item starts as soon as the current one leaves its producer stage, in its own worktree, limited by disk and a shared build cache; (3) one ordered landing queue: each verified candidate is rebased onto the latest main and lands if clean, and goes back to its producer if it conflicts; (4) CI waits and the self-rebuild become background awaits, not blocking steps; (5) how records commits on the working branch stay ordered with several jobs in flight. Give the expected minutes per landed job against today's measured 42-minute median, the disk budget per in-flight job, the failure and conflict handling, and an ordered list of build jobs each sized for one producer run. Add a proposed decision to docs/decisions.md.
> Acceptance:
> - docs/design/async-drain.md covers the five points with the measured baseline, a target in minutes per landed job, the disk budget, conflict handling and an ordered job list.
> - docs/decisions.md has the decision marked proposed.
> - No source file changes; CHANGELOG.md has an entry.
> Files: docs/design/async-drain.md, docs/decisions.md, CHANGELOG.md

## Reading (accepted by the user)

Write a design document for an async, pipelined drain, for Sri and the verifier who judge it. It covers the five requested points against the measured 42-minute median per landed job. Add a proposed decision (next free id, D145) to docs/decisions.md and a CHANGELOG entry; no source changes.

Assumptions:
- Goal: a reviewable design for Sri to accept before any build; audience is Sri and the independent verifier.
- The new decision takes D145, the next free number after D144 in docs/decisions.md; if docs/decision-links.toml needs an entry for it, the producer checks the existing xtask tests and adds one only if a test requires it, else it is out of scope.
- Provenance for the measured numbers is the request's own figures, cited as the user's measurement over 2026-10-27 to 29; the producer must not invent other measurements.
- Related prior decisions the design should cite: D98 (drain), D105 (planning), D112 (per-job build directories), D140 (self-rebuild), D111 (records branch), D46 (per-job directories), D10 (fast-forward landing), D72 and D87 (native PR landing), D79 and D84 (sccache).
- No governance files (athena.toml, defaults.toml, workflows) change; config for the new behavior is proposed in the design only.

## Acceptance criteria

1. [content] docs/design/async-drain.md has a section for each of the five points: read-ahead and hard-job planning; early producer start in its own worktree bounded by disk and shared build cache; one ordered landing queue with rebase-or-return-to-producer; CI waits and self-rebuild as background awaits; ordering of records commits with several jobs in flight.
2. [content] The design states the measured baseline (median 42 minutes per landed job: producer 15, checks 13 to 15, landing about 6 to 8, verifier 2.3, reading and planning up to 5; 35 of 61 jobs landed over 2026-10-27 to 29) and a target in minutes per landed job with a stage-by-stage calculation; any figure not measured is labeled an estimate.
3. [content] The design gives a disk budget per in-flight job (with the arithmetic and the in-flight limit it implies), and failure and conflict handling: rebase conflict returns to the producer, a failed CI, a failed self-rebuild, a crash mid-flight, and a thrash stop.
4. [content] The design lists alternatives considered, and an ordered list of build jobs, each sized for one producer run, with dependencies between them stated.
5. [content] docs/decisions.md has a new entry (next free number after D144, in the same Decision/Alternatives/Tradeoff/Reversal cost/Provenance format) whose heading is marked (proposed) and which links docs/design/async-drain.md.
6. [content] CHANGELOG.md has an entry under Unreleased naming docs/design/async-drain.md and the proposed decision.
7. [behavior] git diff shows changes only to docs/design/async-drain.md, docs/decisions.md and CHANGELOG.md; no file under crates/ or xtask/ changes. Run cargo test and cargo clippy to confirm existing doc-related tests (xtask policy and spec) still pass.

## Expected deliverables

- docs/design/async-drain.md (create)
- docs/decisions.md (modify)
- CHANGELOG.md (modify)
- docs/decision-links.toml (modify)

## Out of scope

- Any source code change in crates/ or xtask/
- Implementing the async drain; only the build job list is written
- Changes to athena.toml, defaults.toml, or workflows
- Changes to records/

## User defaults that apply

- evidence: Claims cite their sources, and numbers are measured or labeled as estimates. Here: Baseline numbers cite the user's measurement; target numbers are labeled estimates.
- designed: Significant designs are written down with alternatives and reviewed before they are built. Here: The design lists alternatives and goes in as a proposed decision for review before building.
- strangers: Code, commits, docs, and records are written for an outside reader. Here: Document written for an outside reader.
- small: Prefer one focused change; size is a design choice, not a wish list. Here: One docs-only change in three files.
- outcome-check: The outcome is the check behind the work, not the activity done. For audience-facing quality, a fresh model reader that took no part in the work rates the live result as a newcomer, and a rating below 4 of 5 blocks completion. Prose-only: no loop code computes the rating yet. Here: Criteria check the target minutes per landed job and the five points are present, not just the file's existence.
- verified: Done means independently verified, and user-facing behavior is verified by exercising it. Here: Verifier runs cargo test and clippy to confirm the docs tests still pass.
- changelog: (not in the defaults model) Here: BRIEF.md asks for tagged releases and a changelog, so the change records itself in CHANGELOG.md.

## Scope

The expected deliverables list is the one statement of scope. A changed file that is not on that list is the scope finding, judged against the list and not against the wording of a criterion.

## Soundness

Would a careful senior engineer reject this approach as a crude heuristic, a special case that should be general, a guess where the code or data could be read, or a rule with obvious false positives (for example, one that matches common words)? Name every such problem, each with the simpler or more correct alternative. Judge the approach only, not style.

A blocker or major soundness finding fails the verdict like an unmet criterion, even when every criterion is met. Minor and nit findings do not.

## Replay of accepted readings

Athena ran the newest accepted readings through the base and the candidate reading rules: 0 of 50 are newly refused by the candidate and were accepted by the base. This is within the allowed share, so the job is not held.

## The change

Base bf52a975a947 → candidate 8fb3525b074c. In your working directory, `git diff HEAD~1` shows it. records/, the owner's private files, and history are absent by design; their absence is not a finding.

```text
 CHANGELOG.md               |   4 +
 docs/decisions.md          |  27 +++++
 docs/design/async-drain.md | 294 +++++++++++++++++++++++++++++++++++++++++++++
 3 files changed, 325 insertions(+)
```

(The diff was inlined for the verifier; it is elided in this record.)

## Checks Athena ran on this tree

- fmt: `cargo fmt --all --check` passed (not run: every changed path matches skip_when_only (docs/, *.md))
- clippy: `cargo clippy --all-targets --locked -- -D warnings` passed (not run: every changed path matches skip_when_only (docs/, *.md))
- test: `cargo test --locked` passed (1634 tests ran)

## Commands Bash may run

- cargo build | check | test | clippy | fmt | run | metadata | tree
- git status | diff | log | show
- ls (paths inside your working directory)

## Verdict

End with the verdict as your structured output: one criteria entry per acceptance criterion, in order.
