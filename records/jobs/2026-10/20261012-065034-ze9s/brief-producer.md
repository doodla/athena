# Job 20261012-065034-ze9s: Add independent review of the study capability M1 design

## Request (verbatim)

> get the study capability design reviewed independently

## Reading (accepted by the user)

Write an independent written review of docs/design/study-capability-m1.md, answering the eight checks in its §9 Review gate, and save it in docs/design/reviews/ like the v0 reviews. The design is not edited and nothing is built.

Assumptions:
- 'Reviewed independently' means producing a written review document of docs/design/study-capability-m1.md per its §9, as the v0 reviews were done, not building anything.
- The producer acts as a reviewer with no shared context with the design's author and does not edit the design to fix findings; answering findings is a later job.
- The review is written to docs/design/reviews/ as a new markdown file.

## Acceptance criteria

1. [content] A new review file exists under docs/design/reviews/ (named like study-capability-m1-review-1.md) that reviews docs/design/study-capability-m1.md.
2. [content] The review addresses each of the eight checks listed in §9 of the design, in order, with a verdict and cited evidence (section or BRIEF.md line) for each.
3. [content] Every finding has a severity (blocker, major, minor, or nit), and the review ends with an overall verdict stating whether the design may proceed to build.
4. [content] The review is written independently of the design's author context, in the style of the existing athena-v0 reviews in docs/design/reviews/, and contains no secrets or personal data.
5. [content] Only the review file (and at most a CHANGELOG entry or a status/pointer line in the design) changes; no Rust code, config, or governance files change.
6. [behavior] Run `git status` and `git diff`: only docs/ files (and optionally CHANGELOG.md) are changed; `cargo fmt --check` still passes.

## Expected deliverables

- docs/design/reviews/study-capability-m1-review-1.md (create)
- CHANGELOG.md (modify)
- docs/design/study-capability-m1.md (modify)

## Out of scope

- Editing the design to answer findings
- Building any part of the study capability
- Studying firstmate
- Changing code, config, or governance files

## User defaults that apply

- evidence: Claims cite their sources, and numbers are measured or labeled as estimates. Here: Review findings cite design sections and BRIEF.md lines.
- designed: Significant designs are written down with alternatives and reviewed before they are built. Here: Design is reviewed before it is built, as the design's review gate requires.
- small: Prefer one focused change; size is a design choice, not a wish list. Here: One new review document; the design itself is not edited.
- strangers: Code, commits, docs, and records are written for an outside reader. Here: Review is written for an outside reader.
- lazy-requests: Requests are short; recover hidden requirements from the repository before asking. Here: Recovered the review checklist from §9 of the design instead of asking.

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

## Report

End with the producer report as your structured output.
