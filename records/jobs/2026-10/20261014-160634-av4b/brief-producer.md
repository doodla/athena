# Job 20261014-160634-av4b: Update the Claude Code pin to 2.1.299

## Request (verbatim)

> Update the Claude Code pin to 2.1.299.

## Reading (accepted by the user)

Change the pinned Claude Code version from 2.1.296 to 2.1.299 in athena.toml (claude_version) and in the current-state docs that quote the pin. Historical review records and test fixtures that merely use a version string are left alone.

Assumptions:
- 2.1.299 is a real published npm release of @anthropic-ai/claude-code; the verifier cannot check this offline.
- Test fixtures using "2.1.296" as an arbitrary version string need not change, as they are self-contained.
- Historical review documents stay at 2.1.296 as dated evidence.
- No checksum or other pin sits beside claude_version that needs updating.

## Acceptance criteria

1. [content] athena.toml sets claude_version = "2.1.299" and no other key in athena.toml changes.
2. [content] README.md and docs/design/athena-v0.md (setup and pin table lines) and docs/decisions.md line about the pin name 2.1.299 as the current pin.
3. [content] Review/acceptance documents under docs/design/reviews/ are unchanged, since they record what was tested at 2.1.296.
4. [behavior] Run `cargo test` and `cargo clippy` and `cargo fmt --check`: all pass with the new pin.

## Expected deliverables

- athena.toml (modify) (a governance file: allowed because the reading lists it)
- README.md (modify)
- docs/design/athena-v0.md (modify)
- docs/decisions.md (modify)
- CHANGELOG.md (modify)
- docs/records.md (modify)
- crates/athena-agent/src/claude.rs (modify)
- crates/athena-gate/src/reconcile.rs (modify)
- crates/athena-gate/src/landing_tests.rs (modify)

## Out of scope

- Rewriting historical review documents
- Changing Claude Code flags or behavior for the new version
- Running live model tests

## User defaults that apply

- lazy-requests: Requests are short; recover hidden requirements from the repository before asking. Here: Found the pin in athena.toml and the docs quoting it by searching the repository.
- small: Prefer one focused change; size is a design choice, not a wish list. Here: One focused version bump; history and fixtures untouched.
- mechanism: Rules that matter are enforced by mechanism, or explicitly marked prose-only. Here: athena.toml is a governance file, so it is a named deliverable and criterion.
- root-cause: Fix the root cause, not the symptom or the one instance that surfaced. The fault is fixed where it originates; when that lies outside the job's scope, name it and propose the fix rather than working around it. The change description names the root cause and why the change addresses it. The root-cause fix is the one focused change. Here: Update every live reference to the pin, not just the config.
- changelog: (not in the defaults model) Here: BRIEF.md asks for tagged releases and a changelog, so the change records itself in CHANGELOG.md.

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
