# Job 20261012-085949-bmgc: Write review 2 of the revised study design, passing policy test

## Request (verbatim)

> try again on 20261012-084057-0eaq

## Reading (accepted by the user)

Re-check the revised study-capability M1 design (docs/design/study-capability-m1.md) against review 1's findings and write it up as review 2. The prior attempt failed only because review 2 used a word the repository's boundary test bans ("operators"); reword so cargo test passes.

Assumptions:
- Review 2 does not exist in the current tree; the earlier attempt's file was discarded, so it must be written again.
- 'Re-check' means a written design review per the design's section 9 gate, not code changes.
- The banned-word list in policy.rs is not to be edited.

## Acceptance criteria

1. [content] docs/design/reviews/study-capability-m1-review-2.md exists, checks each review-1 finding and decision against the revised design, and gives a verdict (approve, or approve with named changes), citing sections.
2. [content] The review contains none of the words banned by the boundary scan in xtask/tests/policy.rs (e.g. 'operators'); it never refers to who runs Athena or where.
3. [content] If the design's status or review-gate section is updated to record review 2, links resolve and no governance file is changed.
4. [behavior] Run `cargo test --locked`: all tests pass, including athena_refers_to_no_one_who_runs_it_or_where.
5. [behavior] Run `cargo fmt --check` and `cargo clippy --locked`: both clean.

## Expected deliverables

- docs/design/reviews/study-capability-m1-review-2.md (create)
- docs/design/study-capability-m1.md (modify)
- CHANGELOG.md (modify)

## Out of scope

- Building the study capability
- Editing policy.rs or any governance file
- Changing records/

## User defaults that apply

- evidence: Claims cite their sources, and numbers are measured or labeled as estimates. Here: Review findings cite design sections and review-1 items.
- strangers: Code, commits, docs, and records are written for an outside reader. Here: Review written for an outside reader.
- tested: Every module has passing tests; cargo test passes and clippy is clean. Here: cargo test must pass, including the boundary policy test.
- small: Prefer one focused change; size is a design choice, not a wish list. Here: One review document, minimal edits elsewhere.

## Checks Athena will run

- fmt: `cargo fmt --all --check`
- clippy: `cargo clippy --all-targets --locked -- -D warnings`
- test: `cargo test --locked`

You may run them first. Athena runs them again on your result, with the base commit's tool configuration.

## Commands Bash may run

- cargo build | check | test | clippy | fmt | run | metadata | tree | doc
- chmod (paths inside your working directory)
- git status | diff | log | show
- ls (paths inside your working directory)
- mv (paths inside your working directory)
- rm (paths inside your working directory)

## Working directory

A snapshot of the base commit's files, with a one-commit git repository of its own. records/, the owner's private files, and history are absent by design. Your edits become the candidate.

## What the verifier sees

The verifier sees only the candidate diff, in a fresh snapshot. It does not see your commands, output, or reasoning. Evidence for a behavior criterion must cite commands the verifier can run itself in that snapshot; your own git status and git diff runs do not count as evidence. Commit hashes made during verification do not exist in the real history, so never pin or cite a commit hash in code, docs, or design-revision text (BRIEF.md included); describe the content instead.

## Report

End with the producer report as your structured output.
