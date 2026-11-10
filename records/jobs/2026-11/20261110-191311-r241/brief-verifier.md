# Job 20261110-191311-r241: verify "Design overlapping CI waits with producer work"

## Request (verbatim)

> Priority: high
> 
> Throughput design: The biggest real wait is each job waiting for CI before its verifier runs: about 15 minutes from dis
> 
> Athena's throughput review 20261110-191047-6szm named the biggest wait in the pipeline and an architectural change that would remove it. Design that change.
> 
> Bottleneck: The biggest real wait is each job waiting for CI before its verifier runs: about 15 minutes from dispatch to the producer's start, likely while the base commit's CI runs, then about 10 minutes for the candidate's CI run. The 2687.2-minute l...
> Stage: checks
> Proposed change: Run the CI waits alongside the model work instead of before it. (1) Start the producer as soon as the job is dispatched; fetch the base's CI result in the background and use it only at the checks. (2) When the producer ends, push the candid...
> Predicted saving: 8.0 minutes per landed job; the median was 2708.1 minutes per landed job when the review ran.
> Why: THROUGHPUT.md puts landing and rebuild at 2687.2 of 2708.1 median minutes. agent-and-build-slots.md §6.1 and per-job-main-landing.md §1 show from the records that most of it is repeated landing listings caused by the stacked local branch: o...
> 
> A change that implements this design must cite throughput review 20261110-191047-6szm in its request text. The next throughput review then sets the minutes it measured after that change landed against this prediction.
> 
> Acceptance:
> - A design note under docs/design/ describes the change and how it removes the bottleneck
> - The note states the minutes per landed job the change should save and how the next throughput review measures them
> 
> Files: docs/design/

## Reading (accepted by the user)

Write a design note under docs/design/ for running the base-CI wait and candidate-CI wait alongside model work instead of before it, citing throughput review 20261110-191047-6szm. Audience: Athena maintainers and the next throughput review, which will score the predicted saving.

Assumptions:
- Existing code: docs/design/agent-and-build-slots.md, docs/design/per-job-main-landing.md, docs/design/continuous-landing.md (existing, not created by this job) are the prior designs the note builds on.
- Goal: remove the dispatch-to-producer CI wait (~15 min) and overlap candidate CI (~10 min) with model work; audience is maintainers and the next throughput review.
- Design only: no code, workflow or config changes; the note is a new file under docs/design/.
- Environment: none reached; the note only describes CI and push behavior and reads only repository files.
- The 8.0 minutes per landed job prediction from the review is used as stated, labeled as an estimate.

## Acceptance criteria

1. [content] A new file under docs/design/ describes starting the producer at dispatch, fetching the base CI result in the background and using it only at the checks, and pushing the candidate early so its CI overlaps verification; Grep for 'base' and 'CI' in it.
2. [content] The design has an Existing code section that reuses or extends docs/design/agent-and-build-slots.md, docs/design/per-job-main-landing.md and docs/design/continuous-landing.md, and states which of their rules change.
3. [content] The note cites throughput review 20261110-191047-6szm by id and states the predicted saving of 8.0 minutes per landed job (labeled an estimate) against the 2708.1 median.
4. [content] The note states how the next throughput review measures the saving: which stage and median minutes per landed job it compares with the prediction after the change lands.
5. [content] The note lists alternatives considered and risks (base CI failing after the producer ran, wasted work), with the handling of each.
6. [content] git diff --stat shows only new or changed files under docs/design/; no crates/, athena.toml, defaults.toml or workflow file changes.

## Expected deliverables

- docs/design/overlap-ci-waits.md (create)
- docs/design/per-job-main-landing.md (modify)
- CHANGELOG.md (modify)

## Out of scope

- Implementing the change in crates/ or xtask/
- Changing athena.toml, defaults.toml or CI workflows
- Changing CHANGELOG.md

## User defaults that apply

- designed: Significant designs are written down with alternatives and reviewed before they are built. Here: Design note includes alternatives and risks.
- evidence: Claims cite their sources, and numbers are measured or labeled as estimates. Here: Numbers cite the review and are labeled estimates.
- outcome-check: The outcome is the check behind the work, not the activity done. For audience-facing quality, a fresh model reader that took no part in the work rates the live result as a newcomer, and a rating below 4 of 5 blocks completion. Prose-only: no loop code computes the rating yet. Here: Note states how the saving is measured.
- changelog: (not in the defaults model) Here: BRIEF.md asks for tagged releases and a changelog, so the change records itself in CHANGELOG.md.

## Scope

The expected deliverables list is the one statement of scope. A changed file that is not on that list is the scope finding, judged against the list and not against the wording of a criterion.

## Soundness

Would a careful senior engineer reject this approach as a crude heuristic, a special case that should be general, a guess where the code or data could be read, or a rule with obvious false positives (for example, one that matches common words)? Name every such problem, each with the simpler or more correct alternative. Judge the approach only, not style.

A blocker or major soundness finding fails the verdict like an unmet criterion, even when every criterion is met. Minor and nit findings do not.

## Replay of accepted readings

Athena ran the newest accepted readings through the base and the candidate reading rules: 0 of 50 are newly refused by the candidate and were accepted by the base. This is within the allowed share, so the job is not held.

## The change

Base 8a0e15cf11d0 → candidate ca79f99b488a. In your working directory, `git diff HEAD~1` shows it. records/, the owner's private files, and history are absent by design; their absence is not a finding.

```text
 CHANGELOG.md                        |   5 ++
 docs/design/overlap-ci-waits.md     | 149 ++++++++++++++++++++++++++++++++++++
 docs/design/per-job-main-landing.md |   3 +
 3 files changed, 157 insertions(+)
```

(The diff was inlined for the verifier; it is elided in this record.)

## Evidence snapshot

Athena copied the real records when this job was dispatched into a read-only directory outside your working copy:

- path: <state>/jobs/20261110-191311-r241/evidence

It holds `records/` (every file under records/ at the base commit), `landings/` (the landing records) and `history.json` (the base commit and the newest 200 first-parent commits of main: hash, commit time, subject, trailers). Every check of real records, such as counting jobs, reading an earlier job's records or comparing a claim with history, uses this snapshot and not the working copy. Read it with the Read, Grep and Glob tools, or run `cargo run -p athena-cli -- status --snapshot <state>/jobs/20261110-191311-r241/evidence`, which prints the job counts, the history summary and the throughput section from it. It is read-only: a write to it fails. The working copy still has no records/.

## Checks Athena ran on this tree

- fmt: `cargo fmt --all --check` passed (not run: every changed path matches skip_when_only (docs/, *.md))
- clippy: `cargo clippy --all-targets --locked -- -D warnings` passed (not run: every changed path matches skip_when_only (docs/, *.md))
- test: `cargo test --locked` passed (result reused from the CI run https://github.com/doodla/athena/actions/runs/42884620092 for ca79f99b488a83850c2008051edaf3126731527f)
- ci-result: `not a check` passed (Check results for ca79f99b488a83850c2008051edaf3126731527f come from the CI run https://github.com/doodla/athena/actions/runs/42884620092 (811s): fmt success, clippy success, test success.)

## Commands Bash may run

- cargo build | check | test | clippy | fmt | run | metadata | tree
- git status | diff | log | show
- ls (paths inside your working directory)

## Verdict

End with the verdict as your structured output: one criteria entry per acceptance criterion, in order.
