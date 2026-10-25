# Job 20261013-024138-xezt: Exempt squash 5a73478 (PR #14) from the history check

## Request (verbatim)

> Retry 20261013-024016-5s8s: exempt 5a734782e76daac9b12bd8833c97a73927c8cf94 (squash of #14; changes records/ but names no job) in [history] exempt. The producer's snapshot has no git history, so use this id.

## Reading (accepted by the user)

Add the full id 5a734782e76daac9b12bd8833c97a73927c8cf94 to `[history] exempt` in athena.toml with a reason comment, and record it as a new decision (D76) in docs/decisions.md, following D70. No code, test, or workflow changes.

Assumptions:
- The user-supplied id is a commit already in main's history, as D60 requires; the snapshot has no git history, so the producer cannot check it.
- The reason is the user's statement: the squash of #14 changes records/ but names no job.
- The next decision number is D76 (the last is D75).
- The exemption entry is appended after f2a8453 in the existing list.

## Acceptance criteria

1. [content] athena.toml `[history] exempt` contains the full 40-character id 5a734782e76daac9b12bd8833c97a73927c8cf94, and all four existing entries remain.
2. [content] A comment above `exempt` says 5a734782e76daac9b12bd8833c97a73927c8cf94 is the squash of PR #14, which changes records/ but names no job.
3. [content] docs/decisions.md has a new D76 after D75 with Decision, Alternatives, Tradeoff, Reversal, and Provenance, stating that the id was supplied by the user and not checked against history.
4. [content] No file other than athena.toml and docs/decisions.md changes; no code, test, or workflow changes.
5. [behavior] Run `cargo test` and `cargo clippy` from the repository root: both pass, including any test that parses athena.toml's exempt list. Run `cargo fmt --check`: it passes.

## Expected deliverables

- athena.toml (modify) (a governance file: allowed because the reading lists it)
- docs/decisions.md (modify)
- CHANGELOG.md (modify)

## Out of scope

- Changing the history check code or its rules
- Rewriting main's history
- Adding the missing job records for #14
- Changing CI workflows

## User defaults that apply

- designed: Significant designs are written down with alternatives and reviewed before they are built. Here: The exemption is written down as decision D76 with alternatives, tradeoff, and reversal, as D70 did.
- mechanism: Rules that matter are enforced by mechanism, or explicitly marked prose-only. Here: Enforced by the existing history check and the D60/D61 validation of exempt entries; no new rule.
- small: Prefer one focused change; size is a design choice, not a wish list. Here: One config entry plus one decision record.
- public: Everything committed is public: no secrets, credentials, or personal data, and transcripts are redacted. Here: Only a commit id and prose are added; no personal data.
- lazy-requests: Requests are short; recover hidden requirements from the repository before asking. Here: Format and wording recovered from D70 and the existing athena.toml comments.

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

The verifier sees only the candidate diff, in a fresh snapshot. It does not see your commands, output, or reasoning. Evidence for a behavior criterion must cite commands the verifier can run itself in that snapshot; your own git status and git diff runs do not count as evidence. Commit hashes made during verification do not exist in the real history, so never pin or cite a commit hash in code, docs, or design-revision text (BRIEF.md included); describe the content instead.

## Report

End with the producer report as your structured output.
