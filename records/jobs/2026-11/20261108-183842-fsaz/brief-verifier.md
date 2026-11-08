# Job 20261108-183842-fsaz: verify "Design per-job landing branches cut from main (review"

## Request (verbatim)

> Priority: high
> 
> Throughput design: The landing and rebuild stage is 1537.1 of the 1582.1-minute median per landed job (THROUGHPUT.md).
> 
> Athena's throughput review 20261108-183648-5eya named the biggest wait in the pipeline and an architectural change that would remove it. Design that change.
> 
> Bottleneck: The landing and rebuild stage is 1537.1 of the 1582.1-minute median per landed job (THROUGHPUT.md). Most of it is very likely not real waiting. Each job's commit keeps riding along in every later pull request, so it shows up in many later l...
> Stage: landing and rebuild
> Proposed change: Change how landing branches are built: one job, one commit, one pull request, cut from the current main. Today landing commits stack on one long-lived local branch, and each pull request carries the whole stack since the last fork from main...
> Predicted saving: 1530.0 minutes per landed job; the median was 1582.1 minutes per landed job when the review ran.
> Why: Measured (THROUGHPUT.md): landing and rebuild is 1537.1 of 1582.1 median minutes per landed job; the other stages together are about 40. The stage ends at "the later of the outcome and the landing record (landings/)". design/agent-and-build...
> 
> A change that implements this design must cite throughput review 20261108-183648-5eya in its request text. The next throughput review then sets the minutes it measured after that change landed against this prediction.
> 
> Acceptance:
> - A design note under docs/design/ describes the change and how it removes the bottleneck
> - The note states the minutes per landed job the change should save and how the next throughput review measures them
> 
> Files: docs/design/

## Reading (accepted by the user)

Write a design note under docs/design/ for throughput review 20261108-183648-5eya: replace the stacked long-lived landing branch with one job, one commit, one pull request cut from current main, so later PRs stop re-carrying earlier commits. Audience: Athena maintainers and the next throughput review.

Assumptions:
- Existing code: crates/athena-git/src/lib.rs, crates/athena-gate/src/lib.rs, crates/athena-gate/src/jobs.rs, crates/athena-data/src/records/throughput.rs, docs/design/continuous-landing.md, docs/design/squash-chain-landing.md, docs/design/native-pr-landing.md and THROUGHPUT.md-related records; the design builds on them rather than from scratch.
- Environment: design-only; no CI, push or PR is run by this job. The note describes GitHub pull request landing; the producer reads only the repository snapshot, with read-only access and no network.
- Goal: remove the stacked-branch re-carrying of commits in later pull requests; audience is maintainers and the next throughput review that scores the prediction.
- The prediction of 1530.0 minutes per landed job is taken from the request; the note labels it an estimate and explains how it is measured.
- The note is a new file under docs/design/; no code, athena.toml or workflow changes are made.
- Term ladder: not used; no structural terms beyond those in the cited design notes.

## Acceptance criteria

1. [content] A new Markdown file under docs/design/ exists in the diff and describes landing each job as one commit in one pull request cut from current main, instead of stacking on a long-lived local branch.
2. [content] The note cites throughput review 20261108-183648-5eya and the measured figures 1537.1 of 1582.1 median minutes (THROUGHPUT.md), checkable by Grep for the review id in the new file.
3. [content] The note states the predicted saving (1530.0 minutes per landed job) labeled as an estimate, and says how the next throughput review measures the minutes after the change landed.
4. [content] The note has an Existing code section that reuses or extends docs/design/continuous-landing.md, docs/design/squash-chain-landing.md and crates/athena-git/src/lib.rs, naming what is kept and what changes.
5. [content] The note lists alternatives considered (for example keep stacking with squash, rebase the stack) with reasons for rejection, and the risks such as conflicts and landing order when cutting from main.
6. [content] The diff touches only files under docs/design/ (git diff --stat), and contains no secrets or personal data.

## Expected deliverables

- docs/design/per-job-landing-branches.md (create)
- docs/design/continuous-landing.md (modify)
- docs/design/squash-chain-landing.md (modify)
- CHANGELOG.md (modify)

## Out of scope

- Implementing the landing change in crates/ or xtask/
- Changes to athena.toml, defaults.toml or workflows
- Editing THROUGHPUT.md or records

## User defaults that apply

- evidence: Claims cite their sources, and numbers are measured or labeled as estimates. Here: Claims cite THROUGHPUT.md and the review; the prediction is labeled an estimate.
- designed: Significant designs are written down with alternatives and reviewed before they are built. Here: The note records alternatives and risks before any build.
- strangers: Code, commits, docs, and records are written for an outside reader. Here: The note is written for an outside reader.
- outcome-check: The outcome is the check behind the work, not the activity done. For audience-facing quality, a fresh model reader that took no part in the work rates the live result as a newcomer, and a rating below 4 of 5 blocks completion. Prose-only: no loop code computes the rating yet. Here: The note names how the next throughput review measures the saving.
- small: Prefer one focused change; size is a design choice, not a wish list. Here: One design note only.
- changelog: (not in the defaults model) Here: BRIEF.md asks for tagged releases and a changelog, so the change records itself in CHANGELOG.md.

## Scope

The expected deliverables list is the one statement of scope. A changed file that is not on that list is the scope finding, judged against the list and not against the wording of a criterion.

## Soundness

Would a careful senior engineer reject this approach as a crude heuristic, a special case that should be general, a guess where the code or data could be read, or a rule with obvious false positives (for example, one that matches common words)? Name every such problem, each with the simpler or more correct alternative. Judge the approach only, not style.

A blocker or major soundness finding fails the verdict like an unmet criterion, even when every criterion is met. Minor and nit findings do not.

## Replay of accepted readings

Athena ran the newest accepted readings through the base and the candidate reading rules: 0 of 50 are newly refused by the candidate and were accepted by the base. This is within the allowed share, so the job is not held.

## The change

Base e76edd60acd8 → candidate d1ef5645ee30. In your working directory, `git diff HEAD~1` shows it. records/, the owner's private files, and history are absent by design; their absence is not a finding.

```text
 docs/design/continuous-landing.md       |   4 +
 docs/design/per-job-landing-branches.md | 192 ++++++++++++++++++++++++++++++++
 docs/design/squash-chain-landing.md     |   4 +
 3 files changed, 200 insertions(+)
```

(The diff was inlined for the verifier; it is elided in this record.)

## Evidence snapshot

Athena copied the real records when this job was dispatched into a read-only directory outside your working copy:

- path: <state>/jobs/20261108-183842-fsaz/evidence

It holds `records/` (every file under records/ at the base commit), `landings/` (the landing records) and `history.json` (the base commit and the newest 200 first-parent commits of main: hash, commit time, subject, trailers). Every check of real records, such as counting jobs, reading an earlier job's records or comparing a claim with history, uses this snapshot and not the working copy. Read it with the Read, Grep and Glob tools, or run `cargo run -p athena-cli -- status --snapshot <state>/jobs/20261108-183842-fsaz/evidence`, which prints the job counts, the history summary and the throughput section from it. It is read-only: a write to it fails. The working copy still has no records/.

## Checks Athena ran on this tree

- fmt: `cargo fmt --all --check` passed (not run: every changed path matches skip_when_only (docs/, *.md))
- clippy: `cargo clippy --all-targets --locked -- -D warnings` passed (not run: every changed path matches skip_when_only (docs/, *.md))
- test: `cargo test --locked` passed (result reused from the CI run https://github.com/doodla/athena/actions/runs/42511349656 for d1ef5645ee30068d38526039e922b1f5a6f07f49)
- ci-result: `not a check` passed (Check results for d1ef5645ee30068d38526039e922b1f5a6f07f49 come from the CI run https://github.com/doodla/athena/actions/runs/42511349656 (565s): fmt success, clippy success, test success.)

## Commands Bash may run

- cargo build | check | test | clippy | fmt | run | metadata | tree
- git status | diff | log | show
- ls (paths inside your working directory)

## Verdict

End with the verdict as your structured output: one criteria entry per acceptance criterion, in order.
