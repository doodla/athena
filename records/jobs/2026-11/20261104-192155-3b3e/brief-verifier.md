# Job 20261104-192155-3b3e: verify "Design continuous drain-owned landing to remove post-verifier wait"

## Request (verbatim)

> Priority: high
> 
> Throughput design: The wait after the verifier finishes and before the landing record exists takes a median of 499.1 of
> 
> Athena's throughput review 20261104-192053-745k named the biggest wait in the pipeline and an architectural change that would remove it. Design that change.
> 
> Bottleneck: The wait after the verifier finishes and before the landing record exists takes a median of 499.1 of the 528.2 minutes per landed job (THROUGHPUT.md, last 20 landed jobs). All other stages together take about 32 minutes. A verified job sits...
> Stage: landing and rebuild
> Proposed change: Make landing one continuous step that Athena owns, run by the drain's ordered landing queue, so a job no longer waits for a later command to notice its merge. When the verifier passes, the queue does the whole sequence in one slot: rebase o...
> Predicted saving: 470.0 minutes per landed job; the median was 528.2 minutes per landed job when the review ran.
> Why: Measured: THROUGHPUT.md gives a median of 528.2 minutes per landed job, of which 499.1 is landing and rebuild. Reading (0.8), planning (4.1), producer (24.2), checks (0.0) and verifier (2.9) add up to about 32, so no change inside those sta...
> 
> A change that implements this design must cite throughput review 20261104-192053-745k in its request text. The next throughput review then sets the minutes it measured after that change landed against this prediction.
> 
> Acceptance:
> - A design note under docs/design/ describes the change and how it removes the bottleneck
> - The note states the minutes per landed job the change should save and how the next throughput review measures them
> 
> Files: docs/design/

## Reading (accepted by the user)

Write a design note under docs/design/ for throughput review 20261104-192053-745k: make landing one continuous step owned by the drain's ordered landing queue, removing the ~499 min median wait between verifier pass and landing record. Audience is the Athena maintainer and the next throughput review.

Assumptions:
- Existing code: docs/design/async-drain.md, docs/design/native-pr-landing.md, docs/design/squash-chain-landing.md, docs/design/parallel-jobs.md, crates/athena-git/src/lib.rs and crates/athena-gate/src/history.rs; the design builds on these and creates none of them.
- Goal: a design note only; no code is implemented here. A later change implementing it must cite review 20261104-192053-745k.
- The request text is truncated; the note takes the quoted bottleneck, 470.0 min predicted saving, and 528.2 / 499.1 baseline figures as given from THROUGHPUT.md.
- No governance files (athena.toml, defaults.toml, workflows) need to change for a design note.

## Acceptance criteria

1. [content] A new file under docs/design/ describes the continuous, drain-owned landing step (rebase, verify, merge, landing record in one queue slot) and explains how it removes the wait between verifier pass and landing record.
2. [content] The design has an Existing code section that reuses or extends docs/design/async-drain.md, docs/design/native-pr-landing.md and crates/athena-git/src/lib.rs, saying what each already provides.
3. [content] The note states the predicted saving of 470.0 minutes per landed job (baseline median 528.2, of which 499.1 is landing and rebuild, citing THROUGHPUT.md) and states how the next throughput review measures the minutes after the change landed.
4. [content] The note cites review 20261104-192053-745k and says an implementing change must cite it in its request text.
5. [content] The note lists alternatives considered, risks (rebase conflicts, ordering, failed landing) and an explicit rollback or failure path.
6. [content] The diff changes only files under docs/design/ (plus optionally CHANGELOG.md); no athena.toml, defaults.toml or workflow file changes.

## Expected deliverables

- docs/design/continuous-landing.md (create)
- docs/design/async-drain.md (modify)
- CHANGELOG.md (modify)

## Out of scope

- Implementing the landing change in crates/
- Changes to athena.toml, defaults.toml or .github/workflows
- Editing THROUGHPUT.md or records/

## User defaults that apply

- designed: Significant designs are written down with alternatives and reviewed before they are built. Here: Note lists alternatives considered and the chosen design.
- evidence: Claims cite their sources, and numbers are measured or labeled as estimates. Here: Numbers cite THROUGHPUT.md and are labeled measured or estimated.
- strangers: Code, commits, docs, and records are written for an outside reader. Here: Note is written for an outside reader.
- outcome-check: The outcome is the check behind the work, not the activity done. For audience-facing quality, a fresh model reader that took no part in the work rates the live result as a newcomer, and a rating below 4 of 5 blocks completion. Prose-only: no loop code computes the rating yet. Here: Criteria check the note states the saving and how the next review measures it.
- small: Prefer one focused change; size is a design choice, not a wish list. Here: One new design document.
- changelog: (not in the defaults model) Here: BRIEF.md asks for tagged releases and a changelog, so the change records itself in CHANGELOG.md.

## Scope

The expected deliverables list is the one statement of scope. A changed file that is not on that list is the scope finding, judged against the list and not against the wording of a criterion.

## Soundness

Would a careful senior engineer reject this approach as a crude heuristic, a special case that should be general, a guess where the code or data could be read, or a rule with obvious false positives (for example, one that matches common words)? Name every such problem, each with the simpler or more correct alternative. Judge the approach only, not style.

A blocker or major soundness finding fails the verdict like an unmet criterion, even when every criterion is met. Minor and nit findings do not.

## Replay of accepted readings

Athena ran the newest accepted readings through the base and the candidate reading rules: 0 of 50 are newly refused by the candidate and were accepted by the base. This is within the allowed share, so the job is not held.

## The change

Base acb5e54bf7b3 → candidate 3a5fe1e6fb52. In your working directory, `git diff HEAD~1` shows it. records/, the owner's private files, and history are absent by design; their absence is not a finding.

```text
 CHANGELOG.md                      |   4 ++
 docs/design/async-drain.md        |   7 +++
 docs/design/continuous-landing.md | 123 ++++++++++++++++++++++++++++++++++++++
 3 files changed, 134 insertions(+)
```

(The diff was inlined for the verifier; it is elided in this record.)

## Checks Athena ran on this tree

- fmt: `cargo fmt --all --check` passed (not run: every changed path matches skip_when_only (docs/, *.md))
- clippy: `cargo clippy --all-targets --locked -- -D warnings` passed (not run: every changed path matches skip_when_only (docs/, *.md))
- test: `cargo test --locked` passed (result reused from the CI run https://github.com/doodla/athena/actions/runs/41917615385 for 3a5fe1e6fb52031c65929812afb742df0753577a)
- ci-result: `not a check` passed (Check results for 3a5fe1e6fb52031c65929812afb742df0753577a come from the CI run https://github.com/doodla/athena/actions/runs/41917615385 (570s): fmt success, clippy success, test success.)

## Commands Bash may run

- cargo build | check | test | clippy | fmt | run | metadata | tree
- git status | diff | log | show
- ls (paths inside your working directory)

## Verdict

End with the verdict as your structured output: one criteria entry per acceptance criterion, in order.
