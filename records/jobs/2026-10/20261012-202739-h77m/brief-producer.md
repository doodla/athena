# Job 20261012-202739-h77m: Write independent review of the native PR landing design

## Request (verbatim)

> review the native PR landing design independently

## Reading (accepted by the user)

Write an independent review of docs/design/native-pr-landing.md as a new file under docs/design/reviews/, checking it against the code and prior landing designs. The design itself is not edited.

Assumptions:
- The user wants a written independent review of docs/design/native-pr-landing.md, in the style of existing reviews under docs/design/reviews/, not a code change.
- The review is a new file numbered review-1 since none exists for this design.
- Review findings are not applied to the design in this job.

## Acceptance criteria

1. [content] docs/design/reviews/native-pr-landing-review-1.md exists and reviews docs/design/native-pr-landing.md independently, citing the design sections and source files (e.g. crates/athena-core/src/land.rs, crates/athena-gate) it checked.
2. [content] The review lists findings with severity, evidence (file and section), and a verdict (approve or changes required), and notes alternatives the design did not consider.
3. [content] Claims are sourced; any number is measured or labeled an estimate.
4. [content] No other files change; the design doc itself is not edited and no governance files change.
5. [behavior] Run `git status` and `git diff`: only the new review file appears.

## Expected deliverables

- docs/design/reviews/native-pr-landing-review-1.md (create)
- docs/design/native-pr-landing.md (modify)
- CHANGELOG.md (modify)

## Out of scope

- Editing the design to address findings
- Implementing native PR landing in code
- Changing governance files

## User defaults that apply

- evidence: Claims cite their sources, and numbers are measured or labeled as estimates. Here: Review findings cite design sections and code locations.
- strangers: Code, commits, docs, and records are written for an outside reader. Here: Review is written for an outside reader.
- small: Prefer one focused change; size is a design choice, not a wish list. Here: One new review document only.
- lazy-requests: Requests are short; recover hidden requirements from the repository before asking. Here: Inferred the target from docs/design/native-pr-landing.md and the reviews directory convention.

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
