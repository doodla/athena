# Job 20261010-132713-ezm8: Show the source commit in `athena --version`

## Request (verbatim)

> athena --version should say which commit the binary was built from, so I can tell which build setup adopted

## Reading (accepted by the user)

`athena --version` currently prints only the Cargo package version, because clap's `version` attribute ignores the commit that `build_info()` already embeds (set by xtask). I read the request as making the version string include the source commit and a dirty/unknown marker.

Assumptions:
- ATHENA_BUILD_COMMIT is already set by `xtask` (xtask/src/main.rs:250) when installing, so only the --version output needs to change; no build.rs is added.
- 'Which build setup adopted' means the installed binary's commit, so a commit hash plus dirty flag is enough.
- The binary is the `athena` bin in crates/athena-cli, whose clap `version` currently prints only CARGO_PKG_VERSION.

## Acceptance criteria

1. [content] `athena --version` output is built from wiring::build_info() and includes the package version and the source commit (short or full hash); clap's default CARGO_PKG_VERSION-only version string is replaced.
2. [content] A build with no embedded commit, or with ATHENA_BUILD_DIRTY not 'false', is shown as such (e.g. 'unknown' and/or 'dirty'), consistent with build_info()'s dirty rule.
3. [content] A unit or integration test covers the version-string formatting for clean, dirty, and unknown-commit builds.
4. [behavior] Run `cargo run -p athena-cli -- --version` (no build env set): it exits 0 and prints the version with an 'unknown' commit marked as dirty/unknown.
5. [behavior] `cargo test`, `cargo clippy` and `cargo fmt --check` pass.

## Expected deliverables

- crates/athena-cli/src/main.rs (modify)
- crates/athena-cli/src/wiring.rs (modify)
- crates/athena-cli/tests/version.rs (create)
- README.md (modify)
- xtask/src/main.rs (modify)

## Out of scope

- Changing how xtask computes the commit or dirty flag
- Adding a build.rs or git-describe dependency
- Changing recorded Build fields in job records

## User defaults that apply

- rust: All code is Rust. Non-Rust files appear only where a platform requires them, and each is listed with its reason. Here: Rust only; no build.rs or scripts added, reusing the existing build_info() and xtask env plumbing.
- tested: Every module has passing tests; cargo test passes and clippy is clean. Here: Add a test for the new version formatting.
- verified: Done means independently verified, and user-facing behavior is verified by exercising it. Here: Verifier runs `athena --version` via cargo run.
- buy-first: Adopt good, maintained tools and crates before building; build only for needs they don't meet, and record the call. Here: Reuse existing build_info() rather than a new git-describe crate or build script.
- small: Prefer one focused change; size is a design choice, not a wish list. Here: One focused change to the version string only.

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

## Report

End with the producer report as your structured output.
