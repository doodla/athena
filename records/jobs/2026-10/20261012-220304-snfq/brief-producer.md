# Job 20261012-220304-snfq: Add [capabilities.study] table to athena.toml

## Request (verbatim)

> split 20261012-214405-xckp: first only add [capabilities.study] so the reader can pick study, and make sure athena.toml still parses; the firstmate pinning is the next job

## Reading (accepted by the user)

Add only an empty-grant [capabilities.study] table to athena.toml, so the reader can pick study and the file still parses. The earlier attempt failed because it added an unknown source_branch field; the firstmate pinning is left for the next job.

Assumptions:
- The study capability is already registered in crates/athena-cli/src/wiring.rs. The reader's choices come from the capabilities that have a config section, so the missing [capabilities.study] table is the only thing keeping the reader from picking it.
- Study sessions need no Bash commands yet, so both command maps are empty. This is the smallest section that parses, because CapabilityConfig requires both fields, denies unknown fields, and defaults env and checks.
- The firstmate pinning (source_branch and the study fetch-pinned work) is out of scope and is the next job.
- The CHANGELOG line that says study has no configured checks or commands stays true after this change, so CHANGELOG.md needs no edit.

## Acceptance criteria

1. [content] athena.toml gains a [capabilities.study] table with producer_commands = {} and verifier_commands = {} (both fields are required by CapabilityConfig), and no other field, such as source_branch or any firstmate pinning.
2. [content] No other section of athena.toml changes, and no Rust source file is modified.
3. [behavior] Run `cargo test --locked`: the tests that load athena.toml pass, so the file still parses and the study capability is registered against its config section.
4. [behavior] Run `cargo clippy --all-targets --locked -- -D warnings` and `cargo fmt --all --check`: both are clean.
5. [content] The athena.toml header comment or a comment on the new table says the table is the study capability's first configuration and that source pinning comes in a later job.

## Expected deliverables

- athena.toml (modify) (a governance file: allowed because the reading lists it)
- CHANGELOG.md (modify)

## Out of scope

- Pinning the study source to a branch or SHA, including the source_branch field and the firstmate pinning (the next job).
- Any change to Rust code, including the study crate and CapabilityConfig.
- Study checks, env, or Bash command grants.
- Fixing the earlier job's source.rs full-clone concern.

## User defaults that apply

- small: Prefer one focused change; size is a design choice, not a wish list. Here: Only the [capabilities.study] table is added, and the pinning work waits for the next job.
- mechanism: Rules that matter are enforced by mechanism, or explicitly marked prose-only. Here: The existing config-parsing tests and the landing gate's 'configuration parses' check enforce that athena.toml stays valid.
- verified: Done means independently verified, and user-facing behavior is verified by exercising it. Here: The verifier runs cargo test, clippy and fmt --check.
- lazy-requests: Requests are short; recover hidden requirements from the repository before asking. Here: I read CapabilityConfig in crates/athena-data/src/config.rs to find the required fields, rather than asking.

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
