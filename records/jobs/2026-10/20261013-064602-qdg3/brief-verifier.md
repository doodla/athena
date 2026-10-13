# Job 20261013-064602-qdg3: verify "Wire sccache warm builds into athena.toml"

## Request (verbatim)

> Wire the sccache warm builds on in athena.toml now that the build supporting it is adopted.

## Reading (accepted by the user)

Add a pinned [tools.sccache] (0.10.0) to athena.toml and set RUSTC_WRAPPER, SCCACHE_DIR and CARGO_INCREMENTAL=0 in the change and study envs, as D79 deferred. Fix the config.rs tests that assumed no pin, and record the change in CHANGELOG and decisions.

Assumptions:
- The pinned version is 0.10.0, which existing tests use.
- Placeholders {athena_home} is supported in env values (per D79); if not, the producer must check paths.rs/env.rs.
- The tools install path layout is {athena_home}/tools/sccache/<version>/bin/sccache as D79 describes.
- A sha256 table is not required for the pin, as the earlier reading's table holds only version.

## Acceptance criteria

1. [content] athena.toml has a [tools.sccache] table holding only `version = "0.10.0"`.
2. [content] [capabilities.change.env] and [capabilities.study.env] each set RUSTC_WRAPPER = "{athena_home}/tools/sccache/0.10.0/bin/sccache" (same version as the pin), SCCACHE_DIR to a path under {athena_home}, and CARGO_INCREMENTAL = "0".
3. [content] The CARGO_TARGET_DIR and CARGO_TERM_COLOR entries in both envs are unchanged.
4. [content] config.rs tests that assumed no sccache pin or appended a second [tools.sccache] table to REPO_CONFIG are rewritten and still check the optional-pin and no-extra-fields behavior.
5. [content] CHANGELOG.md and docs/decisions.md record the wiring, mark D79's deferred step done, and state the speedup is not measured.
6. [behavior] Run `cargo test --locked`: all tests pass, including those that parse athena.toml.
7. [behavior] Run `cargo clippy --all-targets --locked -- -D warnings` and `cargo fmt --all --check`: both clean.

## Expected deliverables

- athena.toml (modify) (a governance file: allowed because the reading lists it)
- crates/athena-data/src/config.rs (modify)
- CHANGELOG.md (modify)
- docs/decisions.md (modify)
- crates/athena-agent/src/env.rs (modify)
- crates/athena-data/src/paths.rs (modify)
- xtask/src/main.rs (modify)
- README.md (modify)

## Out of scope

- Measuring the build speedup.
- Changing the sccache installer or tool-fetching code.
- Changing CARGO_TARGET_DIR or checks.

## User defaults that apply

- lazy-requests: Requests are short; recover hidden requirements from the repository before asking. Here: Recovered the version, env keys and D79 context from the repo and earlier replies.
- mechanism: Rules that matter are enforced by mechanism, or explicitly marked prose-only. Here: The wiring is enforced by config parsing tests.
- tested: Every module has passing tests; cargo test passes and clippy is clean. Here: cargo test and clippy must pass.
- small: Prefer one focused change; size is a design choice, not a wish list. Here: One focused config change with its tests and records.

## The change

Base 0d9b99d1acf5 → candidate 4ec61bf4aba4. In your working directory, `git diff HEAD~1` shows it.

```text
 CHANGELOG.md                     |  5 +++++
 athena.toml                      | 10 ++++++++++
 crates/athena-data/src/config.rs | 29 +++++++++++++++++++++++------
 docs/decisions.md                | 19 ++++++++++++++++++-
 4 files changed, 56 insertions(+), 7 deletions(-)
```

(The diff was inlined for the verifier; it is elided in this record.)

## Checks Athena ran on this tree

- fmt: `cargo fmt --all --check` passed
- clippy: `cargo clippy --all-targets --locked -- -D warnings` passed
- test: `cargo test --locked` passed (321 tests ran)

## Commands Bash may run

- cargo build | check | test | clippy | fmt | run | metadata | tree
- git status | diff | log | show
- ls (paths inside your working directory)

## Verdict

End with the verdict as your structured output: one criteria entry per acceptance criterion, in order.
