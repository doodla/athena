# Job 20261012-180308-knqc: Exempt ee4b0dd from the history check per D66 option (a)

## Request (verbatim)

> per D66, the user chose (a): exempt ee4b0dd1aa6fcd81289dc988d0f7bd38fd7a40ef from the history check

## Reading (accepted by the user)

Add the full 40-character id ee4b0dd1aa6fcd81289dc988d0f7bd38fd7a40ef to `[history] exempt` in athena.toml, as the user chose option (a) in D66. Record the choice in docs/decisions.md and CHANGELOG.md. No code or workflow changes.

Assumptions:
- The user chose (a) alone; (c), the merge-strategy habit, is not implemented here.
- Only the exempt list changes; D66's code analysis stands and no code or tests change.
- The new entry is appended after the two existing entries, with a comment saying why.

## Acceptance criteria

1. [content] athena.toml `[history] exempt` contains exactly the existing two ids plus ee4b0dd1aa6fcd81289dc988d0f7bd38fd7a40ef, as a full 40-character id.
2. [content] The comment above the list explains that ee4b0dd is the multi-job squash of PR #8, excused by the user's choice of D66 (a).
3. [content] docs/decisions.md records the user's choice of (a) for D66, by amending D66 or adding a new entry that states what was decided, the tradeoff, and the reversal (delete the entry). It states that (c) is not yet adopted.
4. [content] CHANGELOG.md notes the new exemption.
5. [behavior] `cargo test` passes and `cargo clippy` is clean, so the existing config-parsing and exemption tests still pass with the new entry.
6. [content] No files other than athena.toml, docs/decisions.md and CHANGELOG.md change. athena.toml is the only governance file touched.

## Expected deliverables

- athena.toml (modify) (a governance file: allowed because the reading lists it)
- docs/decisions.md (modify)
- CHANGELOG.md (modify)

## Out of scope

- Changing `landing` in invariant.rs to accept multi-job commits (option b)
- Setting or enforcing the repository's merge method (option c)
- Rewriting main's history (option d)
- Workflow or code changes

## User defaults that apply

- lazy-requests: Requests are short; recover hidden requirements from the repository before asking. Here: The full commit id came from the request and the cause from D66 in docs/decisions.md, so no question is needed.
- small: Prefer one focused change; size is a design choice, not a wish list. Here: One config entry plus its decision record.
- mechanism: Rules that matter are enforced by mechanism, or explicitly marked prose-only. Here: The existing history check enforces the exemption; D60 requires the full id.
- strangers: Code, commits, docs, and records are written for an outside reader. Here: The comment and decision text explain why the commit is excused.
- public: Everything committed is public: no secrets, credentials, or personal data, and transcripts are redacted. Here: The change adds only a commit id and prose.

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
