# Job 20261013-071915-w56h: Skip sccache install in setup --ci to speed up CI

## Request (verbatim)

> PR CI went from ~4 to ~9 minutes after #24: setup --ci now compiles sccache, which CI never uses. Make CI fast again.

## Reading (accepted by the user)

Setup step 4 builds sccache from source even under `--ci`, and CI never uses it. Skip the sccache install when `--ci` is set, test that, and note it in the docs and changelog.

Assumptions:
- The slowdown is only the sccache `cargo install` in setup step 4, run by `./setup.sh --ci` in both CI jobs; CI sets no RUSTC_WRAPPER.
- The fix is to skip sccache install when --ci is passed, not to remove the pin or add CI caching.
- Other CI speedups (rust-cache, merging jobs) are not requested.

## Acceptance criteria

1. [content] In xtask/src/main.rs, `setup --ci` no longer calls install_sccache; the sccache step prints a skip message when flags.ci is set, and non-CI setup still installs the pinned sccache.
2. [content] The skip decision is a small pure function or a flag check covered by a unit test in xtask (ci skips, non-ci does not).
3. [content] CHANGELOG.md has an entry for the change, and README.md/docs/decisions.md state that --ci skips sccache (README line about sccache setup is corrected).
4. [behavior] Run `cargo test -p xtask` and `cargo clippy --all-targets -- -D warnings` and `cargo fmt --check`: all pass.
5. [content] athena.toml, .github/workflows/ci.yml and the [tools.sccache] pin are unchanged, so the change and study capabilities keep using sccache.

## Expected deliverables

- xtask/src/main.rs (modify)
- CHANGELOG.md (modify)
- README.md (modify)
- docs/decisions.md (modify)

## Out of scope

- Changing the sccache pin or its use by change/study capabilities
- Adding CI dependency caching or restructuring workflow jobs
- Modifying .github/workflows/ci.yml

## User defaults that apply

- small: Prefer one focused change; size is a design choice, not a wish list. Here: Only skip sccache under --ci; no workflow changes.
- tested: Every module has passing tests; cargo test passes and clippy is clean. Here: Add a unit test for the skip decision in xtask.
- lazy-requests: Requests are short; recover hidden requirements from the repository before asking. Here: Located the cause in xtask setup step 4 and the CI workflow rather than asking.
- strangers: Code, commits, docs, and records are written for an outside reader. Here: Changelog and docs note explains why --ci skips sccache.

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
