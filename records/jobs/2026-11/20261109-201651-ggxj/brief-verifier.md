# Job 20261109-201651-ggxj: verify "Design early per-job PRs and dropping the local staging branch"

## Request (verbatim)

> Priority: high
> 
> Throughput design: The landing and rebuild stage takes 1811.5 of the 1859.9-minute median per landed job (THROUGHPUT.md
> 
> Athena's throughput review 20261109-201520-z1xg named the biggest wait in the pipeline and an architectural change that would remove it. Design that change.
> 
> Bottleneck: The landing and rebuild stage takes 1811.5 of the 1859.9-minute median per landed job (THROUGHPUT.md, measured). Most of that is not real waiting. Each job's commit sits on one long-lived local staging branch, and every later pull request c...
> Stage: landing and rebuild
> Proposed change: Open each job's pull request when its producer finishes, not after the verifier, and drop the local staging branch. When the producer ends, Athena builds the job's single landing commit on the host's current main (merge_tree and commit_tree...
> Predicted saving: 7.0 minutes per landed job; the median was 1859.9 minutes per landed job when the review ran.
> Why: What I read: THROUGHPUT.md and these design notes: continuous-landing, async-drain, native-pr-landing, per-job-landing-branches, per-job-main-landing, agent-and-build-slots, warm-baseline-build and ci-routing-baseline. I ran nothing. Where ...
> 
> A change that implements this design must cite throughput review 20261109-201520-z1xg in its request text. The next throughput review then sets the minutes it measured after that change landed against this prediction.
> 
> Acceptance:
> - A design note under docs/design/ describes the change and how it removes the bottleneck
> - The note states the minutes per landed job the change should save and how the next throughput review measures them
> 
> Files: docs/design/

## Reading (accepted by the user)

Write a design note under docs/design/ for opening each job's PR when the producer finishes, building the landing commit on current main, and dropping the long-lived staging branch, to cut the landing and rebuild wait. Audience: Athena maintainers and the next throughput review, which scores the stated saving.

Assumptions:
- Existing code: docs/design/continuous-landing.md, docs/design/native-pr-landing.md, docs/design/per-job-landing-branches.md, docs/design/per-job-main-landing.md, docs/design/async-drain.md, docs/design/agent-and-build-slots.md, docs/design/warm-baseline-build.md and docs/design/ci-routing-baseline.md are existing designs the new note builds on.
- Environment: the design discusses GitHub pull requests and CI; the job itself is docs-only, reads only the repository copy, writes one markdown file, and makes no pushes or network calls.
- Goal: remove the landing and rebuild wait (1811.5 of 1859.9 median minutes per landed job); audience is maintainers and the next throughput review.
- The request text must cite throughput review 20261109-201520-z1xg; the note cites it and THROUGHPUT.md.
- Predicted saving is 7.0 minutes per landed job against a 1859.9-minute median; the note states this and how the next review measures it.
- Only docs are changed; no code, config or governance files, so no code survey or consumers are needed.
- Term staging branch: the long-lived local git branch on which each job's commit sits until its pull request lands.

## Acceptance criteria

1. [content] A new design note exists under docs/design/ that describes opening each job's PR when the producer finishes, building the single landing commit on the host's current main, and dropping the local staging branch.
2. [content] The design has an Existing code section that reuses or extends docs/design/per-job-main-landing.md, docs/design/native-pr-landing.md, docs/design/per-job-landing-branches.md and docs/design/continuous-landing.md, saying what is kept, changed or superseded.
3. [content] The note explains, citing THROUGHPUT.md, how the change removes the landing and rebuild wait (1811.5 of 1859.9 median minutes) and lists alternatives considered.
4. [content] The note states a saving of 7.0 minutes per landed job against the 1859.9-minute median, labeled as a prediction, and names how the next throughput review measures the minutes after the change lands.
5. [content] The note cites throughput review 20261109-201520-z1xg and states that a change implementing it must cite that review in its request text.
6. [content] Only files under docs/design/ are added or changed, per git diff.

## Expected deliverables

- docs/design/early-pr-no-staging-branch.md (create)
- CHANGELOG.md (modify)

## Out of scope

- Implementing the change in crates/ or xtask/
- Editing athena.toml, defaults.toml or workflows
- Editing existing design notes beyond optional cross-links

## User defaults that apply

- evidence: Claims cite their sources, and numbers are measured or labeled as estimates. Here: Numbers cite THROUGHPUT.md or are labeled predictions.
- designed: Significant designs are written down with alternatives and reviewed before they are built. Here: The note lists alternatives.
- outcome-check: The outcome is the check behind the work, not the activity done. For audience-facing quality, a fresh model reader that took no part in the work rates the live result as a newcomer, and a rating below 4 of 5 blocks completion. Prose-only: no loop code computes the rating yet. Here: The note says how the next review measures the saving.
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

Base cff40b5b1197 → candidate e850a243076f. In your working directory, `git diff HEAD~1` shows it. records/, the owner's private files, and history are absent by design; their absence is not a finding.

```text
 CHANGELOG.md                              |   8 ++
 docs/design/early-pr-no-staging-branch.md | 137 ++++++++++++++++++++++++++++++
 2 files changed, 145 insertions(+)
```

(The diff was inlined for the verifier; it is elided in this record.)

## Evidence snapshot

Athena copied the real records when this job was dispatched into a read-only directory outside your working copy:

- path: <state>/jobs/20261109-201651-ggxj/evidence

It holds `records/` (every file under records/ at the base commit), `landings/` (the landing records) and `history.json` (the base commit and the newest 200 first-parent commits of main: hash, commit time, subject, trailers). Every check of real records, such as counting jobs, reading an earlier job's records or comparing a claim with history, uses this snapshot and not the working copy. Read it with the Read, Grep and Glob tools, or run `cargo run -p athena-cli -- status --snapshot <state>/jobs/20261109-201651-ggxj/evidence`, which prints the job counts, the history summary and the throughput section from it. It is read-only: a write to it fails. The working copy still has no records/.

## Checks Athena ran on this tree

- fmt: `cargo fmt --all --check` passed (not run: every changed path matches skip_when_only (docs/, *.md))
- clippy: `cargo clippy --all-targets --locked -- -D warnings` passed (not run: every changed path matches skip_when_only (docs/, *.md))
- test: `cargo test --locked` passed (result reused from the CI run https://github.com/doodla/athena/actions/runs/42692270716 for e850a243076f6d77f3abfd342cb287de4b0d4498)
- ci-result: `not a check` passed (Check results for e850a243076f6d77f3abfd342cb287de4b0d4498 come from the CI run https://github.com/doodla/athena/actions/runs/42692270716 (577s): fmt success, clippy success, test success.)
- producer-report-gaps: `not a check` passed (the producer's report was still incomplete after its send-back, so these were not confirmed by it; judge them yourself from the candidate and the recorded check results: acceptance criteria not met or not reported: "Only files under docs/design/ are added or changed, per git diff.")

## Commands Bash may run

- cargo build | check | test | clippy | fmt | run | metadata | tree
- git status | diff | log | show
- ls (paths inside your working directory)

## Verdict

End with the verdict as your structured output: one criteria entry per acceptance criterion, in order.
