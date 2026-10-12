# Job 20261012-070820-tx19: Review the study capability M1 design in docs/design/reviews

## Request (verbatim)

> try again on 20261012-065034-ze9s

## Reading (accepted by the user)

Retry of the rejected job: write an independent-style review of docs/design/study-capability-m1.md. It goes in docs/design/reviews/ and answers the eight checks in the design's §9 review gate. Also mark the design's status as reviewed, and make sure cargo test, clippy and fmt pass this time.

Assumptions:
- 'Independently reviewed' means a written review by a reviewer who shares no context with the design's author, saved in docs/design/reviews/, as §9 of the design requires.
- The earlier rejection ('check test failed') came from a repository test or policy check, not from the review's content; the retry should run cargo test and fix any docs-related policy failure (for example a link or listing the xtask policy tests read).
- The retry does not build the capability or rewrite the design; findings are recorded and left for the user to answer.

## Acceptance criteria

1. [content] A new file docs/design/reviews/study-capability-m1-review-1.md exists and states a verdict (approve, approve with changes, or reject) on docs/design/study-capability-m1.md, in the style of athena-v0-review-1.md.
2. [content] The review answers each of the eight checks listed in §9 of the design in order, with findings graded by severity and each citing a section of the design or a BRIEF.md line.
3. [content] The review lists decisions for the user as one-line choices and states what it read and any experiments it ran. It makes no claims about firstmate.
4. [content] The design's status line is updated to say that review 1 exists and links to it; no other content of the design changes.
5. [content] No files other than the review, the design doc, and optionally CHANGELOG.md change. No Rust, config, or governance files change, and no personal data appears.
6. [behavior] Run `cargo test` (including xtask policy tests): all pass, including the non-rust-files, architecture and enforcement-map checks that failed or could fail on a docs-only change.
7. [behavior] Run `cargo fmt --check` and `cargo clippy`: both clean.

## Expected deliverables

- docs/design/reviews/study-capability-m1-review-1.md (create)
- docs/design/study-capability-m1.md (modify)
- CHANGELOG.md (modify)
- docs/non-rust-files.md (modify)
- docs/decisions.md (modify)

## Out of scope

- Building any part of the study capability.
- Rewriting the design to answer the review's findings.
- Studying firstmate or making claims about it.
- Changing Rust code, athena.toml, defaults.toml, CI or other governance files.
- Changing records/.

## User defaults that apply

- designed: Significant designs are written down with alternatives and reviewed before they are built. Here: The study capability design must be reviewed before it is built; this job produces that review.
- evidence: Claims cite their sources, and numbers are measured or labeled as estimates. Here: Every review finding cites a design section or a BRIEF.md line, and any numbers are measured or labeled as estimates.
- strangers: Code, commits, docs, and records are written for an outside reader. Here: The review is written for an outside reader, following the format of the existing v0 reviews.
- small: Prefer one focused change; size is a design choice, not a wish list. Here: One new review document plus a one-line status update; no code.
- verified: Done means independently verified, and user-facing behavior is verified by exercising it. Here: cargo test, clippy and fmt --check are run so the earlier test failure does not recur.
- lazy-requests: Requests are short; recover hidden requirements from the repository before asking. Here: The target (docs/design/study-capability-m1.md) and the review format were recovered from the repository.

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
