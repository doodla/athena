# Job 20261013-062226-zmb3: verify "Add sccache support to config and setup; wire athena.toml later"

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

## The change

Base cb5ea9885ffb → candidate 32e74d9f69f7. In your working directory, `git diff HEAD~1` shows it.

```text
 CHANGELOG.md                         |  8 ++++
 README.md                            |  4 +-
 crates/athena-agent/src/env.rs       | 55 +++++++++++++++++++++++--
 crates/athena-core/src/capability.rs |  2 +-
 crates/athena-data/src/config.rs     | 30 +++++++++++++-
 crates/athena-data/src/paths.rs      | 29 ++++++++++++++
 docs/decisions.md                    | 29 +++++++++++++-
 xtask/src/main.rs                    | 78 ++++++++++++++++++++++++++++++++++--
 8 files changed, 223 insertions(+), 12 deletions(-)
```

(The diff was inlined for the verifier; it is elided in this record.)

## Checks Athena ran on this tree

- fmt: `cargo fmt --all --check` passed
- clippy: `cargo clippy --all-targets --locked -- -D warnings` passed
- test: `cargo test --locked` passed (320 tests ran)

## Commands Bash may run

- cargo build | check | test | clippy | fmt | run | metadata | tree
- git status | diff | log | show
- ls (paths inside your working directory)

## Verdict

End with the verdict as your structured output: one criteria entry per acceptance criterion, in order.
