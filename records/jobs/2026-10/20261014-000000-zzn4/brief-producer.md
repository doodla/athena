# Job 20261014-000000-zzn4: Review decision-sets design against revised defaults-loop design

## Request (verbatim)

> Review docs/design/decision-sets.md together with the revised docs/design/defaults-loop.md (its introspection step), written to docs/design/reviews/decision-sets-review-1.md. They must fit as one system: introspection records, the defaults loop, typed decisions in sets, and failure signals that put a decision under review. Check claims against the code and records, check the Jev usage against POST /v1/evaluate as the defaults-loop design now describes it, and say which smallest first build would prove the system on real records. End with a verdict: accept, accept with changes listed, or reject. Review only.

## Reading (accepted by the user)

Write docs/design/reviews/decision-sets-review-1.md, a review-only document. It checks decision-sets.md and the revised defaults-loop.md introspection step as one system, checks claims against code and records, checks Jev usage against POST /v1/evaluate, names the smallest first build, and ends with a verdict.

Assumptions:
- Review style follows the existing files in docs/design/reviews/.
- The revised defaults-loop.md is already committed in the repository.
- POST /v1/evaluate is described only in defaults-loop.md, so it is the reference for the Jev check.
- The review is written as a stranger-readable document with findings ordered by severity.

## Acceptance criteria

1. [content] The review file exists at docs/design/reviews/decision-sets-review-1.md and no other design doc is edited.
2. [content] It covers fit across four parts: introspection records, the defaults loop, typed decisions in sets, and failure signals that put a decision under review. Each gap is stated with a cited file or section.
3. [content] Claims in both designs are checked against named code paths and records, each finding citing its source, with unverified items labeled.
4. [content] A section checks Jev usage against the POST /v1/evaluate contract as the revised defaults-loop.md describes it, listing any mismatches.
5. [content] It names one smallest first build that would prove the system on real records, with a concrete pass/fail observation.
6. [content] It ends with exactly one verdict: accept, accept with changes (listed), or reject.
7. [content] Numbers are measured or labeled as estimates, and the file has no secrets or personal data.
8. [behavior] git status and git diff show only the new review file (and optionally CHANGELOG.md) changed; cargo fmt --check and cargo test still pass.

## Expected deliverables

- docs/design/reviews/decision-sets-review-1.md (create)
- CHANGELOG.md (modify)

## Out of scope

- Editing decision-sets.md or defaults-loop.md
- Implementing the first build
- Changing code, config, or governance files
- Editing records/

## User defaults that apply

- evidence: Claims cite their sources, and numbers are measured or labeled as estimates. Here: Every finding cites the file or code path it was checked against; unmeasured numbers are labeled estimates.
- strangers: Code, commits, docs, and records are written for an outside reader. Here: The review is written for an outside reader.
- small: Prefer one focused change; size is a design choice, not a wish list. Here: One review file only.
- verified: Done means independently verified, and user-facing behavior is verified by exercising it. Here: Findings are checkable by a verifier reading the cited sources.
- lazy-requests: Requests are short; recover hidden requirements from the repository before asking. Here: Review conventions are taken from the existing docs/design/reviews files.

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
