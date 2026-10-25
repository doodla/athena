# Job 20261012-220558-fd7f: Add [capabilities.study] to athena.toml and fix its unit test

## Request (verbatim)

> try again on 20261012-220304-snfq

## Reading (accepted by the user)

Retry of the rejected job: add only a [capabilities.study] section to athena.toml (no firstmate pinning), and update the study crate's test that asserted the section was absent, which failed `cargo test`. athena.toml must still parse and the full checks must pass.

Assumptions:
- The study section mirrors [capabilities.change]: producer_commands, verifier_commands, env, and checks (fmt, clippy, test), since studies are committed files and need the same gates.
- The study capability is already registered in crates/athena-cli/src/wiring.rs; the missing piece is only the configuration section.
- The prior failure was solely the study crate test asserting capability(ID).is_none(); it should now assert the section exists and that grants and checks come from it.
- No firstmate pinning or other edits to athena.toml beyond the new section.

## Acceptance criteria

1. [content] athena.toml gains a [capabilities.study] table (with its env table and checks as needed) and no other existing table changes.
2. [content] The test in crates/athena-cap-study/src/lib.rs no longer asserts the section is absent; it asserts capability("study") is present and that the producer/verifier commands and checks come from it, and the test name no longer says 'nothing_is_configured_yet'.
3. [content] athena.toml is listed as a deliverable and named here as a governance file changed; no firstmate pinning is added.
4. [behavior] Run `cargo test --locked`: all tests pass, including athena-cap-study tests and any test that parses athena.toml.
5. [behavior] Run `cargo clippy --all-targets --locked -- -D warnings` and `cargo fmt --all --check`: both clean.

## Expected deliverables

- athena.toml (modify) (a governance file: allowed because the reading lists it)
- crates/athena-cap-study/src/lib.rs (modify)
- CHANGELOG.md (modify)

## Out of scope

- Firstmate pinning (the next job)
- Changes to the change capability's configuration
- Prompt or rules changes in the study crate
- Other governance files

## User defaults that apply

- small: Prefer one focused change; size is a design choice, not a wish list. Here: Only the study config section and the one test it invalidates.
- tested: Every module has passing tests; cargo test passes and clippy is clean. Here: The study crate test is updated so cargo test and clippy pass.
- mechanism: Rules that matter are enforced by mechanism, or explicitly marked prose-only. Here: The test enforces that the section exists and drives grants and checks.
- verified: Done means independently verified, and user-facing behavior is verified by exercising it. Here: Verifier runs cargo test, clippy, and fmt --check.
- lazy-requests: Requests are short; recover hidden requirements from the repository before asking. Here: The failing test and the existing change section were read from the repository to infer the shape.

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
