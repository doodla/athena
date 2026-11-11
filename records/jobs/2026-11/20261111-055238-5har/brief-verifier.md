# Job 20261111-055238-5har: verify "Add design note: land jobs directly on host main via merge_tree"

## Request (verbatim)

> Priority: high
> 
> Throughput design: Landing and rebuild is 2984.3 of the 2990.6 median minutes per landed job (THROUGHPUT.md). The cause
> 
> Athena's throughput review 20261111-055051-tjym named the biggest wait in the pipeline and an architectural change that would remove it. Design that change.
> 
> Bottleneck: Landing and rebuild is 2984.3 of the 2990.6 median minutes per landed job (THROUGHPUT.md). The cause is the one long-lived local landing branch. Every pull request is cut from it, so it carries all earlier unlanded jobs' commits again, and ...
> Stage: landing and rebuild
> Proposed change: Stop using the local branch as a staging area for landings. When a job's verifier passes, fetch host main and build the job's single landing commit directly on it: a three-way merge with Git::merge_tree (base = the job's recorded base, ours...
> Predicted saving: 5.0 minutes per landed job; the median was 2990.6 minutes per landed job when the review ran.
> Why: Measured in THROUGHPUT.md: landing and rebuild is 2984.3 of the 2990.6 median minutes per landed job; reading, planning, producer, checks and verifier add up to about 28.5. The stage has grown in every review: 440.1 (continuous-landing.md),...
> 
> A change that implements this design must cite throughput review 20261111-055051-tjym in its request text. The next throughput review then sets the minutes it measured after that change landed against this prediction.
> 
> Acceptance:
> - A design note under docs/design/ describes the change and how it removes the bottleneck
> - The note states the minutes per landed job the change should save and how the next throughput review measures them
> 
> Files: docs/design/

## Reading (accepted by the user)

Write a design note under docs/design/ for the change from throughput review 20261111-055051-tjym: stop using the long-lived local landing branch as a staging area, and build each job's single landing commit directly on fetched host main with Git::merge_tree. The note covers the mechanism, alternatives, a predicted saving of 5.0 minutes per landed job, and how the next throughput review measures it. Audience: Athena maintainers who will implement and judge the change.

Assumptions:
- Goal: a design note only; the implementation is a later job. Audience: Athena maintainers and the next throughput review.
- Existing code: crates/athena-git/src/lib.rs (Git::merge_tree), crates/athena-core/src/land.rs, crates/athena-core/src/gh/landing.rs, docs/design/direct-main-landing.md, docs/design/per-job-main-landing.md, docs/design/continuous-landing.md, docs/design/per-job-landing-branches.md.
- Environment: the design involves fetching host main and pushing or opening PRs on the git host; this job only writes docs, with no network or push.
- The request text is truncated; the producer should read THROUGHPUT.md and the cited earlier landing designs to fill details and not invent figures.
- The filename is chosen by the producer; the one in deliverables is my guess.
- Governance files are not changed; no config deliverable, so no Consumers line is needed.

## Acceptance criteria

1. [content] A new note under docs/design/ names throughput review 20261111-055051-tjym and cites THROUGHPUT.md figures (2984.3 of 2990.6 median minutes in landing and rebuild) as evidence.
2. [content] The note explains the mechanism: on verifier pass, fetch host main and build the job's single landing commit on it with Git::merge_tree (base = the job's recorded base). It says why this removes the carried-over unlanded commits that the single long-lived landing branch causes. A reader can check the cause-to-fix link in the text.
3. [content] The note has an 'Existing code:' section naming the existing paths it reuses or extends (crates/athena-git/src/lib.rs, crates/athena-core/src/land.rs, crates/athena-core/src/gh/landing.rs, docs/design/direct-main-landing.md, docs/design/per-job-main-landing.md). It says how it differs from those earlier designs and why they did not remove the wait.
4. [content] The note lists alternatives considered and rejected, conflict handling (merge_tree conflict, host main moving during landing), and rollback or migration from the local branch.
5. [content] The note states the predicted saving of 5.0 minutes per landed job (from a median of 2990.6) and how the next throughput review measures it: the same THROUGHPUT.md landing-and-rebuild median, compared with the prediction, for jobs landed after the change. It also says a change implementing the design must cite review 20261111-055051-tjym in its request text.
6. [content] Only docs (and optionally CHANGELOG.md) change: git diff shows no change to crates/, athena.toml, defaults.toml or .github/workflows/, and cargo build and cargo test still pass unchanged.

## Expected deliverables

- docs/design/direct-main-merge-tree-landing.md (create)
- CHANGELOG.md (modify)
- docs/design/roadmap.md (modify)

## Out of scope

- Implementing the landing change in crates/
- Editing THROUGHPUT.md or records/
- Changing athena.toml, defaults.toml or workflows

## User defaults that apply

- evidence: Claims cite their sources, and numbers are measured or labeled as estimates. Here: The note cites THROUGHPUT.md numbers and labels the 5.0-minute saving as a prediction.
- designed: Significant designs are written down with alternatives and reviewed before they are built. Here: The note includes alternatives and the existing-code reuse.
- outcome-check: The outcome is the check behind the work, not the activity done. For audience-facing quality, a fresh model reader that took no part in the work rates the live result as a newcomer, and a rating below 4 of 5 blocks completion. Prose-only: no loop code computes the rating yet. Here: A criterion checks the measurement method for the next review.
- changelog: (not in the defaults model) Here: BRIEF.md asks for tagged releases and a changelog, so the change records itself in CHANGELOG.md.

## Scope

The expected deliverables list is the one statement of scope. A changed file that is not on that list is the scope finding, judged against the list and not against the wording of a criterion.

## Soundness

Would a careful senior engineer reject this approach as a crude heuristic, a special case that should be general, a guess where the code or data could be read, or a rule with obvious false positives (for example, one that matches common words)? Name every such problem, each with the simpler or more correct alternative. Judge the approach only, not style.

A blocker or major soundness finding fails the verdict like an unmet criterion, even when every criterion is met. Minor and nit findings do not.

## Replay of accepted readings

Athena ran the newest accepted readings through the base and the candidate reading rules: 0 of 50 are newly refused by the candidate and were accepted by the base. This is within the allowed share, so the job is not held.

## The change

Base c8ac6a2ad660 → candidate 71953b38fb85. In your working directory, `git diff HEAD~1` shows it. records/, the owner's private files, and history are absent by design; their absence is not a finding.

```text
 CHANGELOG.md                                  |   5 +
 docs/design/direct-main-merge-tree-landing.md | 167 ++++++++++++++++++++++++++
 docs/design/roadmap.md                        |   3 +
 3 files changed, 175 insertions(+)
```

(The diff was inlined for the verifier; it is elided in this record.)

## Evidence snapshot

Athena copied the real records when this job was dispatched into a read-only directory outside your working copy:

- path: <state>/jobs/20261111-055238-5har/evidence

It holds `records/` (every file under records/ at the base commit), `landings/` (the landing records) and `history.json` (the base commit and the newest 200 first-parent commits of main: hash, commit time, subject, trailers). Every check of real records, such as counting jobs, reading an earlier job's records or comparing a claim with history, uses this snapshot and not the working copy. Read it with the Read, Grep and Glob tools, or run `cargo run -p athena-cli -- status --snapshot <state>/jobs/20261111-055238-5har/evidence`, which prints the job counts, the history summary and the throughput section from it. It is read-only: a write to it fails. The working copy still has no records/.

## Checks Athena ran on this tree

- fmt: `cargo fmt --all --check` passed (not run: every changed path matches skip_when_only (docs/, *.md))
- clippy: `cargo clippy --all-targets --locked -- -D warnings` passed (not run: every changed path matches skip_when_only (docs/, *.md))
- test: `cargo test --locked` passed (result reused from the CI run https://github.com/doodla/athena/actions/runs/42969112238 for 71953b38fb85c3c81134054b7b8b3fe644eb0d3f)
- ci-result: `not a check` passed (Check results for 71953b38fb85c3c81134054b7b8b3fe644eb0d3f come from the CI run https://github.com/doodla/athena/actions/runs/42969112238 (571s): fmt success, clippy success, test success.)

## Commands Bash may run

- cargo build | check | test | clippy | fmt | run | metadata | tree
- git status | diff | log | show
- ls (paths inside your working directory)

## Verdict

End with the verdict as your structured output: one criteria entry per acceptance criterion, in order.
