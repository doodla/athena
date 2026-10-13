# Job 20261013-024016-5s8s: Exempt squash commit 5a73478 from the history check

## Request (verbatim)

> main's history check is red on 5a73478 (the squash of #14). Fix it.

## Reading (accepted by the user)

The history check fails on 5a73478 (squash of #14) because it changes records/ but names no job, intervention or record. The user approved exempting it, so I add its full id to [history].exempt in athena.toml with a comment explaining the cause.

Assumptions:
- The commit is reachable in the repository so its full id can be resolved with git.
- The exemption is for this one commit only; the underlying check logic stays as is.
- The existing rule that a commit adding an exempt entry must already have that commit in its history (D60) is satisfied, since 5a73478 is on main.

## Acceptance criteria

1. [content] athena.toml [history].exempt gains the full 40-character id of the commit abbreviated 5a73478 (resolved with git log/show) and keeps the four existing entries.
2. [content] A comment beside the entry says it is the squash of PR #14, that it changes records/ without naming a job, intervention or record, and that the exemption was approved by the user, following the #8/#9 precedent.
3. [behavior] Run `git show --stat 5a73478` and confirm the id in athena.toml matches that commit in full.
4. [behavior] Run `cargo test`, `cargo clippy --all-targets -- -D warnings` and `cargo fmt --check`; all pass.
5. [content] Among governance files only athena.toml changes, and records/ is untouched.

## Expected deliverables

- athena.toml (modify) (a governance file: allowed because the reading lists it)
- CHANGELOG.md (modify)

## Out of scope

- Changing the history check logic
- Editing records/
- Rewriting main's history
- Changing the other exempt entries

## User defaults that apply

- lazy-requests: Requests are short; recover hidden requirements from the repository before asking. Here: Recovered the precedent for exemptions from the existing [history] comments in athena.toml.
- small: Prefer one focused change; size is a design choice, not a wish list. Here: One-line config change plus a comment.
- strangers: Code, commits, docs, and records are written for an outside reader. Here: The comment explains the cause for an outside reader.
- mechanism: Rules that matter are enforced by mechanism, or explicitly marked prose-only. Here: The exemption goes through the existing config list that the check enforces.

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
