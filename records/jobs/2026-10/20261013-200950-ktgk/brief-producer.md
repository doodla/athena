# Job 20261013-200950-ktgk: Turn on landing.merge_via_rest in athena.toml; fix its test; log it

## Request (verbatim)

> Retry 20261013-195658-8qkc: same change (merge_via_rest = true with the reason comment, and the test checks the default on a stripped config and that this repository turns the merge on). My earlier 'only athena.toml and that test' was too strict: a CHANGELOG.md entry under 'Changed' is wanted too. Nothing else changes.

## Reading (accepted by the user)

Add [landing] merge_via_rest = true to athena.toml with a reason comment. Rewrite the config.rs test to check the default on a config with the [landing] table stripped, and to assert this repository's config turns the merge on. Add one CHANGELOG.md entry under Unreleased 'Changed'.

Assumptions:
- The Unreleased section has no '### Changed' heading yet (the only ones are at lines 58 and 248, in released sections), so the producer adds one below 'Added'.
- The test may strip the table by removing the [landing] section from REPO_CONFIG text, or use a config without it.
- The other existing assertions (explicit true turns it on, empty table is off) stay.

## Acceptance criteria

1. [content] athena.toml has a [landing] table with merge_via_rest = true, preceded by a comment giving the reason.
2. [content] The test in crates/athena-data/src/config.rs derives a stripped config with no [landing] table from REPO_CONFIG and asserts merge_via_rest is false on it.
3. [content] The test asserts that parsing this repository's athena.toml gives merge_via_rest == true, and no longer asserts that REPO_CONFIG lacks [landing].
4. [content] CHANGELOG.md gains an entry under '## [Unreleased]' in a '### Changed' subsection (not 'Added') describing the setting change; no other CHANGELOG lines change.
5. [behavior] Run `cargo test -p athena-data`: rest_merge_is_off_unless_the_config_turns_it_on and the other tests pass.
6. [behavior] Run `cargo clippy --all-targets` and `cargo fmt --check`: both clean.
7. [content] No files other than athena.toml, crates/athena-data/src/config.rs and CHANGELOG.md change.

## Expected deliverables

- athena.toml (modify) (a governance file: allowed because the reading lists it)
- crates/athena-data/src/config.rs (modify)
- CHANGELOG.md (modify)

## Out of scope

- Any other file, including docs, README and workflows
- Changing the default of merge_via_rest in code
- Other settings in athena.toml

## User defaults that apply

- root-cause: Fix the root cause, not the symptom or the one instance that surfaced. The fault is fixed where it originates; when that lies outside the job's scope, name it and propose the fix rather than working around it. The change description names the root cause and why the change addresses it. The root-cause fix is the one focused change. Here: The test pinned the live repo setting instead of the default; it is changed to test the default on a stripped config.
- tested: Every module has passing tests; cargo test passes and clippy is clean. Here: cargo test and clippy must pass.
- small: Prefer one focused change; size is a design choice, not a wish list. Here: One focused change over three files.
- mechanism: Rules that matter are enforced by mechanism, or explicitly marked prose-only. Here: A test enforces both the default and this repo's setting.

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
