# Job 20261111-061100-smxs: verify "Revise merge-tree landing design note: real race detection, scoring"

## Request (verbatim)

> try again on 20261111-055238-5har

## Reading (accepted by the user)

Retry job 20261111-055238-5har: revise the design note for review 20261111-055051-tjym so its race handling rests on real push and host-merge behavior, reconcile the 5.0-minute prediction with the stage it is scored on, and fix roadmap numbering. Audience: Athena maintainers.

Assumptions:
- Retry of job 20261111-055238-5har (rejected: 1 major, 1 minor, 1 nit). Its three changed files (new note, CHANGELOG.md, roadmap.md) are kept as deliverables and revised; each finding is fixed per the criteria.
- Existing code: crates/athena-git/src/lib.rs, crates/athena-core/src/land.rs, crates/athena-core/src/gh/landing.rs, crates/athena-core/src/gh/client.rs, docs/design/direct-main-landing.md, docs/design/per-job-main-landing.md
- Goal: a design note for throughput review 20261111-055051-tjym; audience: Athena maintainers who implement and judge the change.
- Environment: none beyond reading the repository; the note describes pushes and PRs but the job runs no push, reads no records, and writes only docs.
- Before writing, the producer reads the push and merge code and, for every claim about a refusal, conflict or race, names the code path that produces it.
- Design note for a change that does not itself add code; no governance files are edited.

## Acceptance criteria

1. [content] New note docs/design/direct-main-merge-tree-landing.md names throughput review 20261111-055051-tjym and cites the figures 2984.3 of 2990.6 median minutes (landing and rebuild) as evidence, and says which figures come from review text.
2. [content] The note has an Existing code section that reuses or extends crates/athena-git/src/lib.rs, crates/athena-core/src/land.rs, crates/athena-core/src/gh/landing.rs, docs/design/direct-main-landing.md and docs/design/per-job-main-landing.md, and says how it differs from the two earlier designs and why they did not remove the wait.
3. [content] The note explains the mechanism (fetch host main on verifier pass, Git::merge_tree with base = job's recorded base, commit_tree, push as the job's own PR branch) and why it removes carried-over unlanded commits from the single long-lived local landing branch.
4. [content] Fixes finding 1 (major) of job 20261111-055238-5har: the section on host main moving and two jobs landing at once does not claim a push refusal. It states, citing Git::push_branch in crates/athena-git/src/lib.rs (about lines 430-436) that a plain push to a new PR branch is refused only when the branch exists and is not a descendant, so a main move does not fail the push. It names the real detection point: the host merge (gh/client.rs MergeMethod) preceded by an is_ancestor check that the PR's parent is still the main tip (or a host up-to-date requirement), then refetch and rebuild with merge_tree, with a concrete failure signal per case.
5. [content] Fixes finding 2 (minor): the note states which single measured stage figure is scored against the 5.0-minute prediction (the first-record/real-wait part, about 4.6 minutes by the note's own figures, so the prediction is reconciled with that bound), and section on measurement says the headline 2984.3 median is not the one compared with 5.0; it also says a change implementing the design must cite review 20261111-055051-tjym in its request text.
6. [content] The note lists alternatives considered and rejected, merge_tree conflict handling, and rollback or migration from the local branch.
7. [content] Fixes finding 3 (nit): the roadmap item in docs/design/roadmap.md uses plain sequential numbering (no '3b'), and CHANGELOG.md has an entry for the note.
8. [behavior] Only docs change: git diff shows no change under crates/, athena.toml, defaults.toml or .github/workflows/; cargo build and cargo test still pass.

## Expected deliverables

- docs/design/direct-main-merge-tree-landing.md (create)
- CHANGELOG.md (modify)
- docs/design/roadmap.md (modify)

## Out of scope

- Implementing the landing change in crates/
- Editing THROUGHPUT.md or records/
- Changing athena.toml, defaults.toml or workflows

## User defaults that apply

- outcome-check: The outcome is the check behind the work, not the activity done. For audience-facing quality, a fresh model reader that took no part in the work rates the live result as a newcomer, and a rating below 4 of 5 blocks completion. Prose-only: no loop code computes the rating yet. Here: Criteria check the cause-to-fix link and the scored measurement.
- evidence: Claims cite their sources, and numbers are measured or labeled as estimates. Here: Figures cited to sources or labeled as estimates.
- verified: Done means independently verified, and user-facing behavior is verified by exercising it. Here: Verifier checks claims against landing code.
- changelog: (not in the defaults model) Here: BRIEF.md asks for tagged releases and a changelog, so the change records itself in CHANGELOG.md.

## Scope

The expected deliverables list is the one statement of scope. A changed file that is not on that list is the scope finding, judged against the list and not against the wording of a criterion.

## Soundness

Would a careful senior engineer reject this approach as a crude heuristic, a special case that should be general, a guess where the code or data could be read, or a rule with obvious false positives (for example, one that matches common words)? Name every such problem, each with the simpler or more correct alternative. Judge the approach only, not style.

A blocker or major soundness finding fails the verdict like an unmet criterion, even when every criterion is met. Minor and nit findings do not.

## Replay of accepted readings

Athena ran the newest accepted readings through the base and the candidate reading rules: 0 of 50 are newly refused by the candidate and were accepted by the base. This is within the allowed share, so the job is not held.

## The change

Base 79250f582bac → candidate 5bd78f6aed0a. In your working directory, `git diff HEAD~1` shows it. records/, the owner's private files, and history are absent by design; their absence is not a finding.

```text
 CHANGELOG.md                                  |   6 +
 docs/design/direct-main-merge-tree-landing.md | 191 ++++++++++++++++++++++++++
 docs/design/roadmap.md                        |   5 +-
 3 files changed, 201 insertions(+), 1 deletion(-)
```

(The diff was inlined for the verifier; it is elided in this record.)

## Evidence snapshot

Athena copied the real records when this job was dispatched into a read-only directory outside your working copy:

- path: <state>/jobs/20261111-061100-smxs/evidence

It holds `records/` (every file under records/ at the base commit), `landings/` (the landing records) and `history.json` (the base commit and the newest 200 first-parent commits of main: hash, commit time, subject, trailers). Every check of real records, such as counting jobs, reading an earlier job's records or comparing a claim with history, uses this snapshot and not the working copy. Read it with the Read, Grep and Glob tools, or run `cargo run -p athena-cli -- status --snapshot <state>/jobs/20261111-061100-smxs/evidence`, which prints the job counts, the history summary and the throughput section from it. It is read-only: a write to it fails. The working copy still has no records/.

## Checks Athena ran on this tree

- fmt: `cargo fmt --all --check` passed (not run: every changed path matches skip_when_only (docs/, *.md))
- clippy: `cargo clippy --all-targets --locked -- -D warnings` passed (not run: every changed path matches skip_when_only (docs/, *.md))
- test: `cargo test --locked` passed (result reused from the CI run https://github.com/doodla/athena/actions/runs/42970929824 for 5bd78f6aed0ab86176473ba0985ac56086d5015a)
- ci-result: `not a check` passed (Check results for 5bd78f6aed0ab86176473ba0985ac56086d5015a come from the CI run https://github.com/doodla/athena/actions/runs/42970929824 (654s): fmt success, clippy success, test success.)

## Commands Bash may run

- cargo build | check | test | clippy | fmt | run | metadata | tree
- git status | diff | log | show
- ls (paths inside your working directory)

## Verdict

End with the verdict as your structured output: one criteria entry per acceptance criterion, in order.
