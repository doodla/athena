# Job 20261013-131511-k58n: Write review 3 of the native PR landing design

## Request (verbatim)

> Re-review docs/design/native-pr-landing.md independently after its revision for review 2. Review only.

## Reading (accepted by the user)

Write an independent third review of the revised docs/design/native-pr-landing.md as docs/design/reviews/native-pr-landing-review-3.md, in review 2's format. It checks whether review 2's findings are resolved and flags new ones; the design is not edited.

Assumptions:
- The new review file is docs/design/reviews/native-pr-landing-review-3.md, per the user's reply.
- Review 3 is independent of review 2 in verdict but checks review 2's findings against the revision.
- No commands are run to reproduce behavior; evidence is by reading.

## Acceptance criteria

1. [content] docs/design/reviews/native-pr-landing-review-3.md exists, mirrors review 2's structure (header, method, verdict, severity scale, evidence table, numbered findings), and states a clear verdict on the revised design.
2. [content] The review states whether each of review 2's findings (F1–F5) is resolved in the revised design, citing file and line numbers read, and labels claims read/design/reported/unchecked/estimate.
3. [content] The review lists any new findings with severity, and states it was written independently.
4. [content] No file other than the new review is changed; docs/design/native-pr-landing.md is untouched.
5. [content] The review contains no secrets or personal data such as email addresses.

## Expected deliverables

- docs/design/reviews/native-pr-landing-review-3.md (create)
- docs/design/native-pr-landing.md (modify)
- CHANGELOG.md (modify)

## Out of scope

- Editing the design document
- Fixing findings
- Changing records/, code, or governance files

## User defaults that apply

- evidence: Claims cite their sources, and numbers are measured or labeled as estimates. Here: Every claim in the review cites file and lines read or is labeled unchecked/estimate.
- strangers: Code, commits, docs, and records are written for an outside reader. Here: Review is written for an outside reader, like review 2.
- small: Prefer one focused change; size is a design choice, not a wish list. Here: One new file only.
- public: Everything committed is public: no secrets, credentials, or personal data, and transcripts are redacted. Here: No personal data in the review.

## Words you must not write

Athena's boundary scan fails any tracked file that names who runs Athena or the machine it runs on. Do not write these words in any file you touch: code, comments, docs, tests, prompts, changelog or design text. The scan matches whole words, case-insensitively, so capital letters do not help; singular, plural and hyphenated forms are all listed. Reword instead.

Words: `operator`, `operators`, `opctl`, `ccr`, `sandbox`, `sandboxes`, `sandboxed`, `lease`, `leases`, `handoff`, `handoffs`, `hand-off`, `hand-offs`, `cloud session`, `cloud sessions`, `remote session`, `remote sessions`

Fragments (rejected anywhere in a line, even inside a longer word or path): `/home/user`, `claude-0/`, `/root/.`, `.ccr/`, `claude.ai/code`, `session_01`

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

The verifier sees only the candidate diff, in a fresh snapshot. It does not see your commands, output, or reasoning. Evidence for a behavior criterion must cite commands the verifier can run itself in that snapshot; your own git status and git diff runs do not count as evidence. Commit hashes made during verification do not exist in the real history, so never pin or cite a commit hash made during verification in code, docs, or design-revision text (BRIEF.md included); describe the content instead. The one exception is the pinned instance: when the brief has a "Pinned source" section, cite its url, ref and SHA as given.

## Report

End with the producer report as your structured output.
