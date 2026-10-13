# Job 20261013-195040-vy1x: Enable [landing] merge_via_rest in athena.toml

## Request (verbatim)

> Turn on Athena's own merge for this repository: set [landing] merge_via_rest = true in athena.toml. The owner set this repository to squash-only with a ruleset on main (required checks, linear history, no force push) and approved merging verified work once the required checks pass. Change only that setting, with a comment giving the reason.

## Reading (accepted by the user)

Add a [landing] table to athena.toml with merge_via_rest = true and a comment giving the reason (squash-only repo, ruleset on main, owner approved merging verified work once required checks pass). Nothing else changes.

Assumptions:
- athena.toml has no existing [landing] table (grep found none), so one is added, placed after existing tables or near related config.
- The setting is read as config.landing.merge_via_rest (crates/athena-core/src/gh.rs) and defaults to off when absent.

## Acceptance criteria

1. [content] athena.toml contains a [landing] table with merge_via_rest = true, appearing exactly once.
2. [content] A comment directly above the setting states the reason: the repository is squash-only with a ruleset on main (required checks, linear history, no force push), and the owner approved merging verified work once required checks pass.
3. [content] No other line of athena.toml changes, and no other file changes.
4. [behavior] Run `cargo test` and `cargo check`; the config still parses and tests pass. `git diff` shows only the added [landing] lines in athena.toml.

## Expected deliverables

- athena.toml (modify) (a governance file: allowed because the reading lists it)

## Out of scope

- Changing code, docs, CHANGELOG, or CI workflows
- Changing repository settings or the GitHub ruleset
- Other athena.toml settings

## User defaults that apply

- small: Prefer one focused change; size is a design choice, not a wish list. Here: One-setting change in one file.
- public: Everything committed is public: no secrets, credentials, or personal data, and transcripts are redacted. Here: Comment contains no secrets or personal data.
- strangers: Code, commits, docs, and records are written for an outside reader. Here: Comment explains the reason for an outside reader.
- mechanism: Rules that matter are enforced by mechanism, or explicitly marked prose-only. Here: The setting is enforced by code in gh.rs, which reads it.

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
