# Job 20261010-135737-y9gg: Start a CHANGELOG.md in Keep a Changelog format

## Request (verbatim)

> start a changelog

## Reading (accepted by the user)

Create CHANGELOG.md at the repo root in Keep a Changelog format, with an empty Unreleased section and a 0.1.0 section summarizing the initial contents. No tooling or other files change.

Assumptions:
- Format is Keep a Changelog with Semantic Versioning, as the design doc example suggests.
- Workspace version is 0.1.0, so the first released section is 0.1.0, with an empty Unreleased section above it.
- No automated changelog tooling is added; this is a hand-written file.

## Acceptance criteria

1. [content] CHANGELOG.md exists at the repository root in Keep a Changelog form, with an Unreleased section and a 0.1.0 section.
2. [content] The 0.1.0 entries summarize what the repository contains, drawn from README.md, BRIEF.md and docs/architecture.md, with no invented features.
3. [content] CHANGELOG.md contains no secrets or personal data, and no governance files are changed.
4. [behavior] Run git status and git diff: only CHANGELOG.md (and optionally README.md) changed. Run cargo fmt --check and cargo test: both still pass.

## Expected deliverables

- CHANGELOG.md (create)
- README.md (modify)

## Out of scope

- Release tooling, tags, or automated changelog generation
- CI changes
- Rewriting history into per-commit entries

## User defaults that apply

- lazy-requests: Requests are short; recover hidden requirements from the repository before asking. Here: Recovered format and version from Cargo.toml and docs/design/athena-v0.md, which sketches a CHANGELOG.md in Keep a Changelog form.
- small: Prefer one focused change; size is a design choice, not a wish list. Here: One new file, with an optional README link.
- strangers: Code, commits, docs, and records are written for an outside reader. Here: Entries are written for an outside reader.

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
