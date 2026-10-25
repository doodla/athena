# Job 20261012-194212-fmwd: Exempt PR #9 squash f2a8453 from the history check

## Request (verbatim)

> PR #9 was squash-merged as f2a8453380196b95f2e2cc629ffa969af57e6c85 (OP-11); exempt it from the history check like ee4b0dd

## Reading (accepted by the user)

Add the PR #9 squash commit f2a8453380196b95f2e2cc629ffa969af57e6c85 to `[history] exempt` in athena.toml, as D67 did for ee4b0dd. Add a comment, a new decision D70 and a CHANGELOG line. All three give the user's reason: most of PR #9's jobs predate athena.candidate/2, so the chain rule cannot link them. OP-11 is not cited anywhere.

Assumptions:
- The new decision is D70, since D69 is the last entry in docs/decisions.md.
- The user's stated reason replaces any other reason; D69's chain rule requires athena.candidate/2 records, which PR #9's older jobs lack.
- The full id supplied is correct and in history; it cannot be checked from this snapshot.
- The change follows the ee4b0dd precedent in D67 and D68: config, a decision and a changelog line, with no code.

## Acceptance criteria

1. [content] athena.toml `[history] exempt` contains the full id f2a8453380196b95f2e2cc629ffa969af57e6c85 and keeps the three existing entries unchanged.
2. [content] The comment above `exempt` in athena.toml says f2a8453 is the squash of PR #9 and gives the reason: most of PR #9's jobs predate athena.candidate/2, so the chain rule (D69) cannot link them. It does not mention OP-11.
3. [content] docs/decisions.md gains D70 after D69 with decision, alternatives, tradeoff, reversal (delete the entry and comment) and provenance (user request). It cites D60, D67, D68 and D69. It does not mention OP-11.
4. [content] CHANGELOG.md has an entry saying the history check no longer judges f2a8453, with the same reason. It does not mention OP-11.
5. [content] The string OP-11 appears in no changed file.
6. [behavior] Run `cargo test`: all tests pass, including any that parse athena.toml and validate the exempt list.
7. [behavior] Run `git diff --stat` or `git status`: only athena.toml, docs/decisions.md and CHANGELOG.md changed.

## Expected deliverables

- athena.toml (modify) (a governance file: allowed because the reading lists it)
- docs/decisions.md (modify)
- CHANGELOG.md (modify)

## Out of scope

- Any code, test or workflow change
- Restricting merge methods (D66 option c)
- Teaching the gate to link pre-candidate/2 jobs
- Citing OP-11 anywhere

## User defaults that apply

- lazy-requests: Requests are short; recover hidden requirements from the repository before asking. Here: Took the format from the ee4b0dd precedent (D67, D68, the athena.toml comment, CHANGELOG) instead of asking.
- small: Prefer one focused change; size is a design choice, not a wish list. Here: One focused change: a single exempt entry plus its comment, decision and changelog line.
- public: Everything committed is public: no secrets, credentials, or personal data, and transcripts are redacted. Here: The change contains only the commit id and the reason, and no OP-11 reference, as the user asked.
- mechanism: Rules that matter are enforced by mechanism, or explicitly marked prose-only. Here: The existing history check and the D60/D61 exempt validation enforce the entry.

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
