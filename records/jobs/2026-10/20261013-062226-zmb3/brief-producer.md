# Job 20261013-062226-zmb3: Add sccache support to config and setup; wire athena.toml later

## Request (verbatim)

> Retry 20261013-060728-kgp3.

## Reading (accepted by the user)

First of two landings for warm builds: add an optional sccache pin to the config schema, install it in setup with `cargo install --locked` (no hand-copied digests), and support an {athena_home} placeholder in env values. athena.toml is not touched, because the installed gate cannot parse a new field in the same commit; a follow-up job adopts the build and then sets RUSTC_WRAPPER.

Assumptions:
- Per the user's reply, this job lands code and setup only; athena.toml is a later job after the new build is adopted.
- sccache is installed with `cargo install --locked --version <v> --root <athena_home>/tools/sccache/<v>`, so no digests are needed; the user did not choose between this and tarballs, and the no-digest path avoids repeating the placeholder failure.
- The new config field is optional (serde default), so the current athena.toml and the installed gate keep parsing.
- Setup installs sccache only when the pin is present, so setup stays green with today's athena.toml.
- The env placeholder expansion (currently {job_dir} and {side}) lives in crates/athena-core/src/invocation.rs and needs an {athena_home} addition.

## Acceptance criteria

1. [content] crates/athena-data/src/config.rs adds an optional `sccache` entry to Tools holding only a version (no sha256 table), defaulting to absent, so the existing athena.toml still parses.
2. [content] athena.toml, defaults.toml, and other governance files are unchanged; no all-zero or placeholder digest is added anywhere.
3. [content] xtask/src/main.rs setup installs the pinned sccache (when configured) before building Athena, skips it if already installed at tools/sccache/<version>, and fails with a clear message if the install fails.
4. [content] Env value expansion supports an {athena_home} placeholder, and paths.rs derives the sccache binary path from the single pinned version.
5. [content] docs/decisions.md has a new decision with alternatives (tarball with digests, cargo install, no cache) and tradeoffs, marks D46's sccache exclusion superseded, and states that athena.toml adoption follows in a later job; CHANGELOG.md has an entry that claims nothing unverified.
6. [behavior] Run `cargo test --locked`: new tests pass, covering parsing with and without the sccache pin, {athena_home} expansion, and the sccache path derivation.
7. [behavior] Run `cargo clippy --all-targets --locked -- -D warnings` and `cargo fmt --all --check`: both pass.

## Expected deliverables

- crates/athena-data/src/config.rs (modify)
- xtask/src/main.rs (modify)
- crates/athena-data/src/paths.rs (modify)
- crates/athena-core/src/invocation.rs (modify)
- docs/decisions.md (modify)
- CHANGELOG.md (modify)
- setup.sh (modify) (a governance file: allowed because the reading lists it)
- README.md (modify)
- athena.toml (modify) (a governance file: allowed because the reading lists it)

## Out of scope

- Editing athena.toml to set RUSTC_WRAPPER, SCCACHE_DIR, or the sccache pin (follow-up job after the build is adopted).
- Release-tarball installation with sha256 digests.
- Changing per-job CARGO_TARGET_DIR isolation.
- Changing CI workflows or toolchain configuration.

## User defaults that apply

- buy-first: Adopt good, maintained tools and crates before building; build only for needs they don't meet, and record the call. Here: Use the maintained sccache crate instead of building a cache.
- tested: Every module has passing tests; cargo test passes and clippy is clean. Here: Config, placeholder, and path tests; cargo test and clippy must pass.
- evidence: Claims cite their sources, and numbers are measured or labeled as estimates. Here: No claim that a digest was copied from upstream; decision record cites what was measured or labels estimates.
- small: Prefer one focused change; size is a design choice, not a wish list. Here: Only code and setup in this landing; athena.toml deferred.
- mechanism: Rules that matter are enforced by mechanism, or explicitly marked prose-only. Here: A test enforces that the wrapper path derives from the single pinned version.
- rust: All code is Rust. Non-Rust files appear only where a platform requires them, and each is listed with its reason. Here: Installation logic stays in xtask Rust code, not shell.

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
