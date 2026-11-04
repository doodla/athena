# Job 20261029-160024-80qj: verify "Design warm baseline build to cut checks-stage time"

## Request (verbatim)

> Priority: high
> 
> Throughput design: The checks stage (fmt, clippy, test) is the biggest cost: 14.1 of the 25.4-minute median per landed
> 
> Athena's throughput review 20261029-155931-wfkr named the biggest wait in the pipeline and an architectural change that would remove it. Design that change.
> 
> Bottleneck: The checks stage (fmt, clippy, test) is the biggest cost: 14.1 of the 25.4-minute median per landed job, more than twice the producer's 6.8 minutes. The checks run in the verifier's fresh snapshot with an empty per-job, per-side CARGO_TARGE...
> Stage: checks
> Proposed change: Change how the pipeline builds code. Today every check run starts from an empty build directory. Instead, the pipeline keeps one warm baseline build of main, keyed by a hash of rust-toolchain.toml and Cargo.lock, and refreshes it in the bac...
> Predicted saving: 8.0 minutes per landed job; the median was 25.4 minutes per landed job when the review ran.
> Why: I based this on reading THROUGHPUT.md and the design documents; I ran nothing and timed nothing. **What was measured (THROUGHPUT.md):** the median per landed job over the last 20 is 25.4 min. Checks take 14.1, producer 6.8, verifier 2.5, re...
> 
> A change that implements this design must cite throughput review 20261029-155931-wfkr in its request text. The next throughput review then sets the minutes it measured after that change landed against this prediction.
> 
> Acceptance:
> - A design note under docs/design/ describes the change and how it removes the bottleneck
> - The note states the minutes per landed job the change should save and how the next throughput review measures them
> 
> Files: docs/design/

## Reading (accepted by the user)

Write a design note under docs/design/ for a warm baseline build of main (keyed by hash of rust-toolchain.toml and Cargo.lock, refreshed in the background) so checks stop starting from an empty target dir. It cites review 20261029-155931-wfkr, states the 8.0-minute predicted saving and how the next review measures it. rust-toolchain.toml is not changed.

Assumptions:
- Goal: a design note for the warm-baseline build change; audience is Athena's maintainer and the next throughput review, which scores the predicted saving.
- The job is docs-only: it designs the change but does not implement it; implementation would be a later job citing review 20261029-155931-wfkr.
- rust-toolchain.toml is named in the request only as an input to the cache key; this job does not change it.
- Existing code: docs/ design documents and THROUGHPUT.md are read for context; the note is new and creates no code.

## Acceptance criteria

1. [content] A new design note exists under docs/design/ (e.g. docs/design/warm-baseline-build.md) that describes the warm baseline build of main keyed by a hash of rust-toolchain.toml and Cargo.lock, its background refresh, and how checks (fmt, clippy, test) start from it instead of an empty CARGO_TARGET_DIR; checkable by reading the file.
2. [content] The note cites throughput review 20261029-155931-wfkr and the measured figures from THROUGHPUT.md (14.1 of 25.4 minutes median per landed job in checks; producer 6.8), labeling any other number as an estimate.
3. [content] The note states the predicted saving of 8.0 minutes per landed job and names how the next throughput review measures it (the median minutes per landed job over the last 20, with checks-stage minutes compared against 14.1 and 25.4).
4. [content] The note lists alternatives considered (e.g. shared target dir, sccache, no change) and risks such as stale cache and correctness of verification from a warm build, and reviews them.
5. [content] rust-toolchain.toml is only read and cited as the cache key input; git diff shows no change to it, nor to athena.toml, defaults.toml, Cargo.lock or any file outside docs/design/.

## Expected deliverables

- docs/design/warm-baseline-build.md (create)
- CHANGELOG.md (modify)

## Out of scope

- rust-toolchain.toml (read as a cache key input only; must not change)
- Implementing the warm baseline build in crates/ or xtask/
- Changes to athena.toml, defaults.toml, or .github/workflows/

## User defaults that apply

- evidence: Claims cite their sources, and numbers are measured or labeled as estimates. Here: Note cites THROUGHPUT.md figures and labels estimates.
- designed: Significant designs are written down with alternatives and reviewed before they are built. Here: Note includes alternatives and risks.
- small: Prefer one focused change; size is a design choice, not a wish list. Here: One new design note only.
- strangers: Code, commits, docs, and records are written for an outside reader. Here: Written for an outside reader.
- outcome-check: The outcome is the check behind the work, not the activity done. For audience-facing quality, a fresh model reader that took no part in the work rates the live result as a newcomer, and a rating below 4 of 5 blocks completion. Prose-only: no loop code computes the rating yet. Here: Note states how the next review measures the saving.
- changelog: (not in the defaults model) Here: BRIEF.md asks for tagged releases and a changelog, so the change records itself in CHANGELOG.md.

## Scope

The expected deliverables list is the one statement of scope. A changed file that is not on that list is the scope finding, judged against the list and not against the wording of a criterion.

## Soundness

Would a careful senior engineer reject this approach as a crude heuristic, a special case that should be general, a guess where the code or data could be read, or a rule with obvious false positives (for example, one that matches common words)? Name every such problem, each with the simpler or more correct alternative. Judge the approach only, not style.

A blocker or major soundness finding fails the verdict like an unmet criterion, even when every criterion is met. Minor and nit findings do not.

## Replay of accepted readings

Athena ran the newest accepted readings through the base and the candidate reading rules: 0 of 50 are newly refused by the candidate and were accepted by the base. This is within the allowed share, so the job is not held.

## The change

Base 2f0487ac03bd → candidate fbb174e64ba7. In your working directory, `git diff HEAD~1` shows it. records/, the owner's private files, and history are absent by design; their absence is not a finding.

```text
 CHANGELOG.md                       |   7 +++
 docs/design/warm-baseline-build.md | 117 +++++++++++++++++++++++++++++++++++++
 2 files changed, 124 insertions(+)
```

(The diff was inlined for the verifier; it is elided in this record.)

## Checks Athena ran on this tree

- fmt: `cargo fmt --all --check` passed (not run: every changed path matches skip_when_only (docs/, *.md))
- clippy: `cargo clippy --all-targets --locked -- -D warnings` passed (not run: every changed path matches skip_when_only (docs/, *.md))
- test: `cargo test --locked` passed (result reused from the CI run https://github.com/doodla/athena/actions/runs/41829411253 for fbb174e64ba7abe0c1267e592b842a827c639b60)
- ci-result: `not a check` passed (Check results for fbb174e64ba7abe0c1267e592b842a827c639b60 come from the CI run https://github.com/doodla/athena/actions/runs/41829411253 (463s): fmt success, clippy success, test success.)

## Commands Bash may run

- cargo build | check | test | clippy | fmt | run | metadata | tree
- git status | diff | log | show
- ls (paths inside your working directory)

## Verdict

End with the verdict as your structured output: one criteria entry per acceptance criterion, in order.
