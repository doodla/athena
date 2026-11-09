# Job 20261109-055645-r796: verify "Design removing the local landing branch from landing"

## Request (verbatim)

> Priority: high
> 
> Throughput design: Landing and rebuild takes 1996.9 of the 2063.6 median minutes per landed job (THROUGHPUT.md, last 20
> 
> Athena's throughput review 20261109-055516-tzwd named the biggest wait in the pipeline and an architectural change that would remove it. Design that change.
> 
> Bottleneck: Landing and rebuild takes 1996.9 of the 2063.6 median minutes per landed job (THROUGHPUT.md, last 20 landings); the other five stages together take about 37. The design notes show that almost all of this time is not real waiting. Every veri...
> Stage: landing and rebuild
> Proposed change: Remove the local landing branch from the pipeline. Today the path is: verifier, then a fast-forward onto a shared local branch, then a pull request cut from that stack. Instead, take each verified job straight from the verifier to its own p...
> Predicted saving: 5.0 minutes per landed job; the median was 2063.6 minutes per landed job when the review ran.
> Why: Sources: THROUGHPUT.md and the design notes; nothing was re-measured here. THROUGHPUT.md: the median is 2063.6 minutes per landed job, of which landing and rebuild is 1996.9. This stage has grown with each review: 440.1 minutes (continuous-...
> 
> A change that implements this design must cite throughput review 20261109-055516-tzwd in its request text. The next throughput review then sets the minutes it measured after that change landed against this prediction.
> 
> Acceptance:
> - A design note under docs/design/ describes the change and how it removes the bottleneck
> - The note states the minutes per landed job the change should save and how the next throughput review measures them
> 
> Files: docs/design/

## Reading (accepted by the user)

Write a design note under docs/design/ for taking each verified job straight to its own pull request, removing the shared local landing branch. It cites throughput review 20261109-055516-tzwd, states the 5.0 min/job predicted saving, and says how the next review measures it.

Assumptions:
- Goal and audience: the maintainer and the next throughput review, who judge whether the design removes the landing-and-rebuild wait.
- Existing code: crates/athena-core/src/land.rs, crates/athena-core/src/pr.rs, docs/design/continuous-landing.md, docs/design/per-job-main-landing.md, docs/design/native-pr-landing.md, docs/design/squash-chain-landing.md, docs/design/per-job-landing-branches.md (all exist; the job creates only a new note).
- Environment: the note discusses pull requests but nothing is pushed or run; read-only repository docs and code, no network or CI reached.
- Design only: no code, config or workflow changes; the note is a new file docs/design/ named for the change, e.g. per-job-pr-landing.md.
- Numbers (1996.9 of 2063.6 minutes, 5.0 predicted saving) are taken from the request and THROUGHPUT.md; the note labels them as not re-measured.

## Acceptance criteria

1. [content] A new file under docs/design/ describes taking each verified job straight to its own pull request without the shared local landing branch, and explains how that removes the landing-and-rebuild wait (Grep for 'landing branch' and 'pull request' in the new file).
2. [content] The note states the predicted saving of 5.0 minutes per landed job against the 2063.6 median, and says how the next throughput review measures it (Grep for '5.0' and 'next throughput review').
3. [content] The note cites throughput review 20261109-055516-tzwd (Grep for that id in the new file).
4. [content] The design has an Existing code section that reuses or extends crates/athena-core/src/land.rs, crates/athena-core/src/pr.rs and docs/design/continuous-landing.md, saying what each contributes.
5. [content] The note lists alternatives considered (e.g. docs/design/per-job-main-landing.md, squash-chain-landing.md) with reasons for rejecting them, and cites THROUGHPUT.md for measured numbers or labels them as estimates.
6. [content] git diff shows only added files under docs/design/; no other paths change.

## Expected deliverables

- docs/design/per-job-pr-landing.md (create)
- CHANGELOG.md (modify)
- docs/design/roadmap.md (modify)

## Out of scope

- Implementing the change in crates/ code
- Changes to athena.toml, defaults.toml or workflows
- Re-measuring throughput

## User defaults that apply

- designed: Significant designs are written down with alternatives and reviewed before they are built. Here: Note records alternatives and is a design doc to be reviewed before building.
- evidence: Claims cite their sources, and numbers are measured or labeled as estimates. Here: Numbers cited to THROUGHPUT.md or labeled estimates.
- outcome-check: The outcome is the check behind the work, not the activity done. For audience-facing quality, a fresh model reader that took no part in the work rates the live result as a newcomer, and a rating below 4 of 5 blocks completion. Prose-only: no loop code computes the rating yet. Here: Criteria require the saving and its measurement by the next review.
- changelog: (not in the defaults model) Here: BRIEF.md asks for tagged releases and a changelog, so the change records itself in CHANGELOG.md.

## Scope

The expected deliverables list is the one statement of scope. A changed file that is not on that list is the scope finding, judged against the list and not against the wording of a criterion.

## Soundness

Would a careful senior engineer reject this approach as a crude heuristic, a special case that should be general, a guess where the code or data could be read, or a rule with obvious false positives (for example, one that matches common words)? Name every such problem, each with the simpler or more correct alternative. Judge the approach only, not style.

A blocker or major soundness finding fails the verdict like an unmet criterion, even when every criterion is met. Minor and nit findings do not.

## Replay of accepted readings

Athena ran the newest accepted readings through the base and the candidate reading rules: 0 of 50 are newly refused by the candidate and were accepted by the base. This is within the allowed share, so the job is not held.

## The change

Base a773e3eed13b → candidate f62a67667c06. In your working directory, `git diff HEAD~1` shows it. records/, the owner's private files, and history are absent by design; their absence is not a finding.

```text
 docs/design/per-job-pr-landing.md | 99 +++++++++++++++++++++++++++++++++++++++
 1 file changed, 99 insertions(+)
```

(The diff was inlined for the verifier; it is elided in this record.)

## Evidence snapshot

Athena copied the real records when this job was dispatched into a read-only directory outside your working copy:

- path: <state>/jobs/20261109-055645-r796/evidence

It holds `records/` (every file under records/ at the base commit), `landings/` (the landing records) and `history.json` (the base commit and the newest 200 first-parent commits of main: hash, commit time, subject, trailers). Every check of real records, such as counting jobs, reading an earlier job's records or comparing a claim with history, uses this snapshot and not the working copy. Read it with the Read, Grep and Glob tools, or run `cargo run -p athena-cli -- status --snapshot <state>/jobs/20261109-055645-r796/evidence`, which prints the job counts, the history summary and the throughput section from it. It is read-only: a write to it fails. The working copy still has no records/.

## Checks Athena ran on this tree

- fmt: `cargo fmt --all --check` passed (not run: every changed path matches skip_when_only (docs/, *.md))
- clippy: `cargo clippy --all-targets --locked -- -D warnings` passed (not run: every changed path matches skip_when_only (docs/, *.md))
- test: `cargo test --locked` passed (result reused from the CI run https://github.com/doodla/athena/actions/runs/42570765291 for f62a67667c061eb267052281680f2434546d360b)
- ci-result: `not a check` passed (Check results for f62a67667c061eb267052281680f2434546d360b come from the CI run https://github.com/doodla/athena/actions/runs/42570765291 (467s): fmt success, clippy success, test success.)

## Commands Bash may run

- cargo build | check | test | clippy | fmt | run | metadata | tree
- git status | diff | log | show
- ls (paths inside your working directory)

## Verdict

End with the verdict as your structured output: one criteria entry per acceptance criterion, in order.
