# Job 20261029-135024-p78m: verify "Design async drain without assuming sccache; add D145 proposed"

## Request (verbatim)

> try again on 20261029-073414-yg89

## Reading (accepted by the user)

Retry of job 20261029-073414-yg89: write docs/design/async-drain.md (five points, measured 42-minute baseline, target, disk budget, conflict handling, ordered build jobs) for Sri and the verifier, with the sccache premise fixed per D84, plus proposed decision D145 and a CHANGELOG entry.

Assumptions:
- Job 20261029-073414-yg89 findings addressed: (major) sccache treated as active although D84 removed it: the retry states the cache is off, recomputes the disk budget and recheck estimate without it, marks cache-dependent numbers conditional, and adds an early build job to find the 0-hit cause and measure hits across per-job worktrees; re-enabling is a separate costed option. (minor) job 10, not job 7, raises max_in_flight; (minor) max_check_slots added to the [drain.pipeline] block; (minor) the target is computed per landed job, with failed jobs' check-slot use worked in using 35 of 61 landed.
- The previous candidate's three changed files (docs/design/async-drain.md, docs/decisions.md, CHANGELOG.md) are kept as deliverables but rewritten from base since attempts are discarded; the retry starts from base.
- docs/decision-links.toml was listed earlier; it is kept as a possible deliverable only if a check requires a link entry for D145. Governance files athena.toml, defaults.toml and workflows are untouched.
- The doc includes a 'Premises checked' section listing each dependency (build cache, drain, producer, verifier, disk guard, config limits) with the repo path or decision showing its current state, produced by reading the repo and docs/decisions.md before writing.
- Next free decision id is D145 (after D144); the entry uses the Decision/Alternatives/Tradeoff/Reversal cost/Provenance format.

## Acceptance criteria

1. [content] docs/design/async-drain.md has a section for each of the five points: read-ahead and hard-job planning; early producer start in own worktree bounded by disk; one ordered landing queue with rebase-or-return-to-producer; CI waits and self-rebuild as background awaits; ordering of records commits with several jobs in flight.
2. [content] Grep of docs/design/async-drain.md for sccache finds no statement treating it as active; each mention says D84 removed it or labels it a proposed option, and a 'Premises checked' section lists each dependency (build cache, drain, producer, verifier, disk guard, config limits) with a repo path or decision id.
3. [content] The design states the measured baseline (median 42 minutes; producer 15, checks 13 to 15, landing 6 to 8, verifier 2.3, reading and planning up to 5; 35 of 61 landed over 2026-10-27 to 29) and a target in minutes per landed job with stage-by-stage arithmetic that includes failed jobs' use of the check slot; unmeasured or cache-dependent figures are labeled estimate or conditional.
4. [content] The disk budget per in-flight job is computed without a shared cache, with the implied in-flight limit; the [drain.pipeline] config block includes max_in_flight, max_check_slots, read_ahead and reserve_disk_gb; the text names job 10 as the one that raises max_in_flight.
5. [content] Failure and conflict handling covers rebase conflict returning to the producer, failed CI, failed self-rebuild, crash mid-flight and thrash stop; alternatives are listed; ordered build jobs each sized for one producer run state dependencies, with an early job measuring cache hits across worktrees.
6. [content] docs/decisions.md has D145 with heading marked (proposed), in the existing format, linking docs/design/async-drain.md; CHANGELOG.md has an Unreleased entry naming the design doc and the proposed decision.
7. [behavior] Run git diff --stat: only docs/design/async-drain.md, docs/decisions.md, CHANGELOG.md (and docs/decision-links.toml if needed) change; nothing under crates/ or xtask/. Run cargo test and cargo clippy and confirm they pass.

## Expected deliverables

- docs/design/async-drain.md (create)
- docs/decisions.md (modify)
- CHANGELOG.md (modify)
- docs/decision-links.toml (modify)

## Out of scope

- Any source code change in crates/ or xtask/
- Implementing the async drain; only the build job list is written
- Changes to athena.toml, defaults.toml, or workflows (including re-enabling sccache)
- Changes to records/

## User defaults that apply

- evidence: Claims cite their sources, and numbers are measured or labeled as estimates. Here: Baseline numbers cited from the request; unmeasured figures labeled estimate or conditional.
- designed: Significant designs are written down with alternatives and reviewed before they are built. Here: Design doc lists alternatives and a premise check before build jobs.
- outcome-check: The outcome is the check behind the work, not the activity done. For audience-facing quality, a fresh model reader that took no part in the work rates the live result as a newcomer, and a rating below 4 of 5 blocks completion. Prose-only: no loop code computes the rating yet. Here: Criteria check the per-landed-job minutes target and conflict behavior, not mere file existence.
- small: Prefer one focused change; size is a design choice, not a wish list. Here: One focused docs change.
- strangers: Code, commits, docs, and records are written for an outside reader. Here: Doc written for outside readers.
- tested: Every module has passing tests; cargo test passes and clippy is clean. Here: cargo test and clippy confirmed to still pass.
- changelog: (not in the defaults model) Here: BRIEF.md asks for tagged releases and a changelog, so the change records itself in CHANGELOG.md.

## Scope

The expected deliverables list is the one statement of scope. A changed file that is not on that list is the scope finding, judged against the list and not against the wording of a criterion.

## Soundness

Would a careful senior engineer reject this approach as a crude heuristic, a special case that should be general, a guess where the code or data could be read, or a rule with obvious false positives (for example, one that matches common words)? Name every such problem, each with the simpler or more correct alternative. Judge the approach only, not style.

A blocker or major soundness finding fails the verdict like an unmet criterion, even when every criterion is met. Minor and nit findings do not.

## Replay of accepted readings

Athena ran the newest accepted readings through the base and the candidate reading rules: 0 of 50 are newly refused by the candidate and were accepted by the base. This is within the allowed share, so the job is not held.

## The change

Base 9cb590b7441a → candidate 0f8dba703763. In your working directory, `git diff HEAD~1` shows it. records/, the owner's private files, and history are absent by design; their absence is not a finding.

```text
 CHANGELOG.md               |   5 +
 docs/decisions.md          |  23 +++++
 docs/design/async-drain.md | 231 +++++++++++++++++++++++++++++++++++++++++++++
 3 files changed, 259 insertions(+)
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
