# Job 20261110-025121-wmxr: verify "Design note: land each job directly on host main, not a local branch"

## Request (verbatim)

> Priority: high
> 
> Throughput design: The landing-and-rebuild stage: 2203.6 of the 2252.0-minute median per landed job (THROUGHPUT.md), up
> 
> Athena's throughput review 20261110-024838-mch9 named the biggest wait in the pipeline and an architectural change that would remove it. Design that change.
> 
> Bottleneck: The landing-and-rebuild stage: 2203.6 of the 2252.0-minute median per landed job (THROUGHPUT.md), up in each review (1069.2, 1537.1, 1842.1, now 2203.6). Its cause is the single long-lived local landing branch: each pull request is cut from...
> Stage: landing and rebuild
> Proposed change: Stop using the local branch as a staging area. When a job's verifier passes, build its single landing commit directly on the host's current main (three-way merge from the job's recorded base: Git::merge_tree, then Git::commit_tree), push it...
> Predicted saving: 5.0 minutes per landed job; the median was 2252.0 minutes per landed job when the review ran.
> Why: Measured (THROUGHPUT.md): landing and rebuild is 2203.6 of 2252.0 median minutes per landed job; the other five stages add up to about 35. The stage ends at the later of the outcome and the newest landing record listing the job (THROUGHPUT....
> 
> A change that implements this design must cite throughput review 20261110-024838-mch9 in its request text. The next throughput review then sets the minutes it measured after that change landed against this prediction.
> 
> Acceptance:
> - A design note under docs/design/ describes the change and how it removes the bottleneck
> - The note states the minutes per landed job the change should save and how the next throughput review measures them
> 
> Files: docs/design/

## Reading (accepted by the user)

Write a design note under docs/design/ for throughput review 20261110-024838-mch9: stop using the long-lived local landing branch as a staging area. Build each verified job's landing commit directly on host main (merge_tree then commit_tree) and push it. The note states the predicted saving of 5.0 minutes per landed job and how the next throughput review measures it. Audience: Athena maintainers and the next throughput review.

Assumptions:
- Goal: a written design (no code) for maintainers and the next throughput review. The request's truncated text is completed from its stated proposed change.
- Existing code: docs/design/per-job-main-landing.md, docs/design/per-job-landing-branches.md, docs/design/continuous-landing.md, crates/athena-git/src/lib.rs (merge_tree, commit_tree), crates/athena-core/src/land.rs; the note extends these and does not rewrite them.
- Environment: the note describes push and PR behavior on the host's GitHub remote but exercises nothing; this job only edits docs, with no network, no permissions and no data reached.
- Term ladder: not used; no structural ladder terms are introduced.
- Producer should read THROUGHPUT.md if present for the quoted numbers and label anything unmeasured as an estimate.
- Consumers: none found; no config files change.

## Acceptance criteria

1. [content] A new file under docs/design/ cites throughput review 20261110-024838-mch9 and quotes the measured 2203.6 of 2252.0 median minutes per landed job from THROUGHPUT.md.
2. [content] The note explains how the change removes the bottleneck: the verified job's single landing commit is built on current host main via Git::merge_tree from the job's recorded base, then Git::commit_tree, then pushed, so the local landing branch is no longer a staging area. It says why the stage's wait (the later of the outcome and the newest landing record) shrinks.
3. [content] The note states the predicted saving of 5.0 minutes per landed job and says how the next throughput review measures it: it compares the landing-and-rebuild minutes measured after the change landed with this prediction.
4. [content] The note has alternatives considered, risks and rollback sections. It has an Existing code section that reuses or extends docs/design/per-job-main-landing.md, docs/design/per-job-landing-branches.md, crates/athena-git/src/lib.rs and crates/athena-core/src/land.rs by path, and states what differs from them.
5. [content] The note says that a change implementing the design must cite throughput review 20261110-024838-mch9 in its request text.

## Expected deliverables

- docs/design/direct-main-landing.md (create)
- CHANGELOG.md (modify)

## Out of scope

- Implementing the change in crates/
- Editing athena.toml, defaults.toml or workflows
- Editing existing design notes beyond a link

## User defaults that apply

- evidence: Claims cite their sources, and numbers are measured or labeled as estimates. Here: Numbers are cited from THROUGHPUT.md or labeled as estimates.
- designed: Significant designs are written down with alternatives and reviewed before they are built. Here: The note includes alternatives and risks.
- outcome-check: The outcome is the check behind the work, not the activity done. For audience-facing quality, a fresh model reader that took no part in the work rates the live result as a newcomer, and a rating below 4 of 5 blocks completion. Prose-only: no loop code computes the rating yet. Here: A criterion requires the measurement method against the 5.0 minute prediction.
- changelog: (not in the defaults model) Here: BRIEF.md asks for tagged releases and a changelog, so the change records itself in CHANGELOG.md.

## Scope

The expected deliverables list is the one statement of scope. A changed file that is not on that list is the scope finding, judged against the list and not against the wording of a criterion.

## Soundness

Would a careful senior engineer reject this approach as a crude heuristic, a special case that should be general, a guess where the code or data could be read, or a rule with obvious false positives (for example, one that matches common words)? Name every such problem, each with the simpler or more correct alternative. Judge the approach only, not style.

A blocker or major soundness finding fails the verdict like an unmet criterion, even when every criterion is met. Minor and nit findings do not.

## Replay of accepted readings

Athena ran the newest accepted readings through the base and the candidate reading rules: 0 of 50 are newly refused by the candidate and were accepted by the base. This is within the allowed share, so the job is not held.

## The change

Base 02255b555192 → candidate b0c1b5762dab. In your working directory, `git diff HEAD~1` shows it. records/, the owner's private files, and history are absent by design; their absence is not a finding.

```text
 CHANGELOG.md                       |   4 ++
 docs/design/direct-main-landing.md | 112 +++++++++++++++++++++++++++++++++++++
 2 files changed, 116 insertions(+)
```

(The diff was inlined for the verifier; it is elided in this record.)

## Evidence snapshot

Athena copied the real records when this job was dispatched into a read-only directory outside your working copy:

- path: <state>/jobs/20261110-025121-wmxr/evidence

It holds `records/` (every file under records/ at the base commit), `landings/` (the landing records) and `history.json` (the base commit and the newest 200 first-parent commits of main: hash, commit time, subject, trailers). Every check of real records, such as counting jobs, reading an earlier job's records or comparing a claim with history, uses this snapshot and not the working copy. Read it with the Read, Grep and Glob tools, or run `cargo run -p athena-cli -- status --snapshot <state>/jobs/20261110-025121-wmxr/evidence`, which prints the job counts, the history summary and the throughput section from it. It is read-only: a write to it fails. The working copy still has no records/.

## Checks Athena ran on this tree

- fmt: `cargo fmt --all --check` passed (not run: every changed path matches skip_when_only (docs/, *.md))
- clippy: `cargo clippy --all-targets --locked -- -D warnings` passed (not run: every changed path matches skip_when_only (docs/, *.md))
- test: `cargo test --locked` passed (result reused from the CI run https://github.com/doodla/athena/actions/runs/42746226522 for b0c1b5762dab269bb59218fb233bd7cdcd9c3249)
- ci-result: `not a check` passed (Check results for b0c1b5762dab269bb59218fb233bd7cdcd9c3249 come from the CI run https://github.com/doodla/athena/actions/runs/42746226522 (672s): fmt success, clippy success, test success.)

## Commands Bash may run

- cargo build | check | test | clippy | fmt | run | metadata | tree
- git status | diff | log | show
- ls (paths inside your working directory)

## Verdict

End with the verdict as your structured output: one criteria entry per acceptance criterion, in order.
