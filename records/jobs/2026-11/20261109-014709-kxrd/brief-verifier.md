# Job 20261109-014709-kxrd: verify "Design per-job landing from current main to remove the landing wait"

## Request (verbatim)

> Priority: high
> 
> Throughput design: The landing-and-rebuild stage is 1842.1 of the 1903.5 median minutes per landed job (THROUGHPUT.md).
> 
> Athena's throughput review 20261109-014617-5cg0 named the biggest wait in the pipeline and an architectural change that would remove it. Design that change.
> 
> Bottleneck: The landing-and-rebuild stage is 1842.1 of the 1903.5 median minutes per landed job (THROUGHPUT.md). The other five stages add up to about 38 minutes. The likely cause: landing commits pile up on one long-lived local branch, so each job's c...
> Stage: landing and rebuild
> Proposed change: Stop stacking landings on one branch. Use one job, one commit, one pull request, cut from current main. When a job's verifier passes, fetch the host's main and build the job's single landing commit directly on it with a three-way merge from...
> Predicted saving: 1830.0 minutes per landed job; the median was 1903.5 minutes per landed job when the review ran.
> Why: Evidence I read: (1) THROUGHPUT.md: landing and rebuild is 1842.1 of 1903.5 median minutes. The other stages are about 38 minutes (0.4 + 11.0 + 22.5 + 1.0 + 3.3). (2) agent-and-build-slots.md section 6.1 (measured on the records): job b4f5'...
> 
> A change that implements this design must cite throughput review 20261109-014617-5cg0 in its request text. The next throughput review then sets the minutes it measured after that change landed against this prediction.
> 
> Acceptance:
> - A design note under docs/design/ describes the change and how it removes the bottleneck
> - The note states the minutes per landed job the change should save and how the next throughput review measures them
> 
> Files: docs/design/

## Reading (accepted by the user)

Write a design note under docs/design/ for throughput review 20261109-014617-5cg0: stop stacking landings on one local branch; build each job's single commit on fetched main via three-way merge and open one PR. The audience is Athena's maintainers and the next throughput review, which scores the prediction.

Assumptions:
- Existing code: docs/design/per-job-landing-branches.md, docs/design/squash-chain-landing.md, docs/design/native-pr-landing.md, docs/design/continuous-landing.md, docs/design/agent-and-build-slots.md, crates/athena-core/src/land.rs and crates/athena-core/src/status/throughput.rs are read and not created by this job.
- Environment: the job runs locally with read access to the repository copy; it writes one markdown file and does not push, open PRs or read records/. The design describes fetching host main and opening a PR but does not do so. THROUGHPUT.md and records are absent from the copy, so figures are cited from the request text and labeled as quoted.
- Goal: a reviewable design that removes the 1842.1 min landing wait; audience is maintainers and the next throughput review.
- Term ladder: not used; no structural terms beyond those quoted from the request.
- Consumers: none found; no config files change.
- Prior designs on landing exist; the note must say how it differs from or supersedes them rather than repeat them.

## Acceptance criteria

1. [content] A new file under docs/design/ exists (checked with ls docs/design) and cites throughput review 20261109-014617-5cg0 and the figures 1842.1 and 1903.5 median minutes.
2. [content] The note has an Existing code section that reuses or extends docs/design/per-job-landing-branches.md, docs/design/squash-chain-landing.md, docs/design/native-pr-landing.md and crates/athena-core/src/land.rs, saying what is kept and what changes.
3. [content] The note explains the mechanism: one job, one commit, one PR cut from current main; fetch host main after verifier pass and build the single landing commit with a three-way merge; and why this removes the wait on a stacked branch.
4. [content] The note lists alternatives considered with reasons for rejection, and failure cases (merge conflict, main moved, verified content differing).
5. [content] The note states the predicted saving of 1830.0 minutes per landed job and how the next throughput review measures it, naming the landing-and-rebuild stage in crates/athena-core/src/status/throughput.rs and comparing against the 1903.5 median.
6. [content] The note states that an implementing change must cite review 20261109-014617-5cg0 in its request text.
7. [content] Only files under docs/design/ change (git diff --stat); no governance files, records/ or code change, and CHANGELOG.md is not touched.

## Expected deliverables

- docs/design/per-job-main-landing.md (create)
- docs/design/per-job-landing-branches.md (modify)
- docs/decisions.md (modify)
- CHANGELOG.md (modify)

## Out of scope

- Implementing the change in crates/
- Editing athena.toml, defaults.toml or workflows
- Editing THROUGHPUT.md or records/

## User defaults that apply

- strangers: Code, commits, docs, and records are written for an outside reader. Here: Note written for an outside reader with definitions.
- evidence: Claims cite their sources, and numbers are measured or labeled as estimates. Here: Numbers cited to THROUGHPUT.md and the review or labeled estimates.
- designed: Significant designs are written down with alternatives and reviewed before they are built. Here: Design includes alternatives and is written before any build.
- outcome-check: The outcome is the check behind the work, not the activity done. For audience-facing quality, a fresh model reader that took no part in the work rates the live result as a newcomer, and a rating below 4 of 5 blocks completion. Prose-only: no loop code computes the rating yet. Here: Criteria check the measured saving against the next review.
- small: Prefer one focused change; size is a design choice, not a wish list. Here: One design note only.
- public: Everything committed is public: no secrets, credentials, or personal data, and transcripts are redacted. Here: No secrets or personal data in the note.
- changelog: (not in the defaults model) Here: BRIEF.md asks for tagged releases and a changelog, so the change records itself in CHANGELOG.md.

## Scope

The expected deliverables list is the one statement of scope. A changed file that is not on that list is the scope finding, judged against the list and not against the wording of a criterion.

## Soundness

Would a careful senior engineer reject this approach as a crude heuristic, a special case that should be general, a guess where the code or data could be read, or a rule with obvious false positives (for example, one that matches common words)? Name every such problem, each with the simpler or more correct alternative. Judge the approach only, not style.

A blocker or major soundness finding fails the verdict like an unmet criterion, even when every criterion is met. Minor and nit findings do not.

## Replay of accepted readings

Athena ran the newest accepted readings through the base and the candidate reading rules: 0 of 50 are newly refused by the candidate and were accepted by the base. This is within the allowed share, so the job is not held.

## The change

Base f17fd2709733 → candidate 4d82199af38a. In your working directory, `git diff HEAD~1` shows it. records/, the owner's private files, and history are absent by design; their absence is not a finding.

```text
 docs/design/per-job-main-landing.md | 127 ++++++++++++++++++++++++++++++++++++
 1 file changed, 127 insertions(+)
```

(The diff was inlined for the verifier; it is elided in this record.)

## Evidence snapshot

Athena copied the real records when this job was dispatched into a read-only directory outside your working copy:

- path: <state>/jobs/20261109-014709-kxrd/evidence

It holds `records/` (every file under records/ at the base commit), `landings/` (the landing records) and `history.json` (the base commit and the newest 200 first-parent commits of main: hash, commit time, subject, trailers). Every check of real records, such as counting jobs, reading an earlier job's records or comparing a claim with history, uses this snapshot and not the working copy. Read it with the Read, Grep and Glob tools, or run `cargo run -p athena-cli -- status --snapshot <state>/jobs/20261109-014709-kxrd/evidence`, which prints the job counts, the history summary and the throughput section from it. It is read-only: a write to it fails. The working copy still has no records/.

## Checks Athena ran on this tree

- fmt: `cargo fmt --all --check` passed (not run: every changed path matches skip_when_only (docs/, *.md))
- clippy: `cargo clippy --all-targets --locked -- -D warnings` passed (not run: every changed path matches skip_when_only (docs/, *.md))
- test: `cargo test --locked` passed (result reused from the CI run https://github.com/doodla/athena/actions/runs/42547873077 for 4d82199af38abd02ec2ebc2133da51b56371a5e2)
- ci-result: `not a check` passed (Check results for 4d82199af38abd02ec2ebc2133da51b56371a5e2 come from the CI run https://github.com/doodla/athena/actions/runs/42547873077 (594s): fmt success, clippy success, test success.)

## Commands Bash may run

- cargo build | check | test | clippy | fmt | run | metadata | tree
- git status | diff | log | show
- ls (paths inside your working directory)

## Verdict

End with the verdict as your structured output: one criteria entry per acceptance criterion, in order.
