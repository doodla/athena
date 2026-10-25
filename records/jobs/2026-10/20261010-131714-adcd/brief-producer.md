# Job 20261010-131714-adcd: Show the build commit in `athena --version` via build_info()

## Request (verbatim)

> athena --version should say which commit the binary was built from, so I can tell which build setup adopted

## Reading (accepted by the user)

Make `athena --version` print the package version, the source commit and a dirty marker from the existing `wiring::build_info()`, which setup fills from ATHENA_BUILD_COMMIT and ATHENA_BUILD_DIRTY. This replaces clap's default version string, which shows only the package version. There is no build.rs. A test covers the output.

Assumptions:
- Setup/xtask already export ATHENA_BUILD_COMMIT and ATHENA_BUILD_DIRTY (xtask/src/main.rs lines 250-251), so nothing there changes.
- clap's `version` attribute currently shows only CARGO_PKG_VERSION; it must be replaced with a runtime string built from build_info().
- Commit is shown in short form (first 7 characters) if xtask passes the full hash; the formatter handles either.
- Dirty marker is shown as `, dirty`; unknown commit prints `unknown`.

## Acceptance criteria

1. [behavior] `athena --version` (and `-V`) prints the package version, the short source commit from build_info(), and a dirty marker when build_info().dirty is true, e.g. `athena 0.1.0 (abc1234, dirty)`. Check by building with ATHENA_BUILD_COMMIT=abc1234 ATHENA_BUILD_DIRTY=false and running the binary.
2. [behavior] A build with no ATHENA_BUILD_COMMIT prints `unknown` as the commit, and is marked dirty (existing build_info() semantics). Check by running a plain `cargo build -p athena-cli` binary.
3. [content] The text comes from wiring::build_info() (a small formatting function there or in lib.rs, wired into the clap `version` attribute); no build.rs is added, and build_info()'s existing environment handling is unchanged.
4. [behavior] A test in crates/athena-cli/tests/binary.rs runs `athena --version` and asserts the output contains the package version and either a hex commit or `unknown`; a unit test covers the formatting for clean, dirty and unknown cases. `cargo test`, `cargo clippy` and `cargo fmt --check` pass.
5. [content] No new non-Rust files; no governance files change.

## Expected deliverables

- crates/athena-cli/src/main.rs (modify)
- crates/athena-cli/src/wiring.rs (modify)
- crates/athena-cli/tests/binary.rs (modify)
- crates/athena-cli/src/lib.rs (modify)
- docs/architecture.md (modify)

## Out of scope

- Adding a build.rs or the vergen crate
- Changing xtask or setup.sh
- Changing build_info()'s environment variable handling
- Changing what is recorded in job records

## User defaults that apply

- rust: All code is Rust. Non-Rust files appear only where a platform requires them, and each is listed with its reason. Here: Change and tests are Rust only.
- tested: Every module has passing tests; cargo test passes and clippy is clean. Here: Adds a binary test and a formatting unit test; clippy and fmt must be clean.
- small: Prefer one focused change; size is a design choice, not a wish list. Here: Touches only the CLI's version string, reusing build_info().
- verified: Done means independently verified, and user-facing behavior is verified by exercising it. Here: Verifier runs the built binary with and without the build env vars.
- lazy-requests: Requests are short; recover hidden requirements from the repository before asking. Here: Recovered the existing build_info() and the xtask env from the repository.

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
