# Job 20261013-054813-6nrv: Enable skip_when_only for fmt and clippy checks in athena.toml

## Request (verbatim)

> Turn on the prose-only check skip in athena.toml now that the build supporting it is adopted.

## Reading (accepted by the user)

Add skip_when_only = ["docs/", "*.md"] to the change capability's fmt and clippy checks in athena.toml. The test check keeps running on every change, because the xtask policy tests read docs/*.md and scan every tracked file, including .md files, for boundary terms. Update the CHANGELOG, which currently says enabling is a follow-up.

Assumptions:
- The user's reply is read as: tests must not be skipped for files a test reads. I checked xtask/tests/policy.rs: it reads docs/non-rust-files.md, docs/architecture.md and docs/enforcement.md, and the boundary scan reads every tracked file outside records/, including all .md files. So no prose pattern is safe for the test check, and I leave it unskipped.
- Clippy and fmt do not read prose, so skipping them on docs/ and *.md is safe. I did not check whether any crate uses include_str! on a .md file.
- The pattern syntax is from path_matches in crates/athena-core/src/checks.rs: 'dir/' matches a directory, '*.ext' matches any depth.

## Acceptance criteria

1. [content] In athena.toml, the [[capabilities.change.checks]] entries named fmt and clippy each have skip_when_only = ["docs/", "*.md"], and no other patterns.
2. [content] The change capability's test check has no skip_when_only key, and athena.toml has a comment explaining why: xtask/tests/policy.rs reads docs/*.md and scans every tracked file, so a prose change can fail a test.
3. [content] The [capabilities.study] checks and every other athena.toml key are unchanged.
4. [content] CHANGELOG.md under Unreleased says the key is now enabled for fmt and clippy only, and the earlier 'does not set the key' follow-up wording no longer reads as pending.
5. [behavior] Run `cargo test --locked`, `cargo clippy --all-targets --locked -- -D warnings` and `cargo fmt --all --check`; all pass. The repository's config tests load athena.toml, so this shows it still parses.

## Expected deliverables

- athena.toml (modify) (a governance file: allowed because the reading lists it)
- CHANGELOG.md (modify)
- docs/enforcement.md (modify)

## Out of scope

- The study capability's checks
- Changing the skip mechanism in Rust code
- Skipping the test check for any path
- records/ and the owner's files

## User defaults that apply

- small: Prefer one focused change; size is a design choice, not a wish list. Here: One focused config change plus a changelog note.
- mechanism: Rules that matter are enforced by mechanism, or explicitly marked prose-only. Here: The skip is enforced by the runner's skip_note logic, which already has tests.
- strangers: Code, commits, docs, and records are written for an outside reader. Here: The athena.toml comment says what the key does and why the test check is excluded.
- verified: Done means independently verified, and user-facing behavior is verified by exercising it. Here: The verifier runs test, clippy and fmt --check.

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
