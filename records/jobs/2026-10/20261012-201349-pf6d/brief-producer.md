# Job 20261012-201349-pf6d: Design native PR landing behind a code-host interface

## Request (verbatim)

> design how Athena lands its own work through PRs natively (see the 2026-10-12 brief addenda)

## Reading (accepted by the user)

Write docs/design/native-pr-landing.md designing how Athena pushes, opens, tracks and merges its own PRs per the 2026-10-12 addenda. All code-host operations sit behind a host-neutral interface, with GitHub as the first adapter only. Add a decisions.md entry; no code.

Assumptions:
- Design only; no implementation, no trait code committed (the interface is specified in prose/signature sketches in the doc).
- Credentials for the host adapter are handled by the host CLI/environment, never stored in files.
- Review by a separate reviewer is requested after the doc, per the designed default, but review files are not part of this change.

## Acceptance criteria

1. [content] docs/design/native-pr-landing.md exists with status, problem, alternatives (at least: Athena opens/merges PRs in core via a code-host interface; keep operator-driven PRs; hybrid), recommendation, and implementation plan; claims are marked read/inferred with file/line citations.
2. [content] The design defines a code-host interface (a Rust trait: push branch, open PR, read status/checks, read comments, merge, close) with GitHub as the first adapter only. It lists the operations, a host-neutral vocabulary (change request rather than PR where practical), and what a second adapter (e.g. GitLab) would need. Nothing in core names GitHub types.
3. [content] It cites the 2026-10-12 addenda in BRIEF.md ('Athena does what the operator should not have to'; 'Brownfield integration' stealth requirement) and states how each is met.
4. [content] It covers the lifecycle: branch push, PR open, title/body from reading and verdict, CI wait, review comments as untrusted data, merge method including squash chains per docs/design/squash-chain-landing.md, and failure/rollback.
5. [content] It states BRIEF.md constraints: only this repo, never push or open PRs upstream; third-party content untrusted; stealth mode adds no Athena files to PRs against third-party repos.
6. [content] It names the mechanism enforcing each rule or marks it prose-only, and lists open questions and review needs.
7. [content] docs/decisions.md gains one entry recording the design call, including the host-interface decision. No code, tests, or governance files change.
8. [behavior] Run `git status` and `git diff --stat`: only the design doc (and optionally decisions.md/CHANGELOG.md) changed; no .rs, Cargo, athena.toml, defaults.toml, or workflow files.

## Expected deliverables

- docs/design/native-pr-landing.md (create)
- docs/decisions.md (modify)
- CHANGELOG.md (modify)
- docs/architecture.md (modify)

## Out of scope

- Any implementation, Rust code, or tests
- Adapters other than GitHub (only the interface is designed)
- Changes to governance files, CI workflows, or athena.toml
- Pushing or opening PRs against upstream or third-party repos

## User defaults that apply

- designed: Significant designs are written down with alternatives and reviewed before they are built. Here: Doc with alternatives, recommendation, open questions, review needs before any build.
- evidence: Claims cite their sources, and numbers are measured or labeled as estimates. Here: Claims cite BRIEF.md and repo files with read/inferred labels.
- mechanism: Rules that matter are enforced by mechanism, or explicitly marked prose-only. Here: Each rule names its enforcing mechanism or is marked prose-only.
- small: Prefer one focused change; size is a design choice, not a wish list. Here: One doc plus one decisions entry.
- public: Everything committed is public: no secrets, credentials, or personal data, and transcripts are redacted. Here: No secrets or personal data in the doc.
- strangers: Code, commits, docs, and records are written for an outside reader. Here: Written for an outside reader.

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
