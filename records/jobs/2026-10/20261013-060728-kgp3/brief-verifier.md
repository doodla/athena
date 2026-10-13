# Job 20261013-060728-kgp3: verify "Add pinned sccache for warm builds while keeping per-job target dirs"

## Request (verbatim)

> Jobs are slow because every job builds from cold: the test check takes ~165s per job, mostly compiling, since each job's CARGO_TARGET_DIR starts empty (D46). Make builds warm without giving up job isolation.

## Reading (accepted by the user)

Add sccache (pinned, checksummed install via xtask setup like gitleaks) with RUSTC_WRAPPER, a shared SCCACHE_DIR, and CARGO_INCREMENTAL=0 in athena.toml, keeping per-job CARGO_TARGET_DIR. Record a decision amending D46 and update the design text that says every job builds cold.

Assumptions:
- sccache is installed to the same tool path scheme as gitleaks, and RUSTC_WRAPPER can reference it by a path or PATH entry that jobs receive.
- Checksums for the pinned sccache release are taken from the official release assets; the producer cannot verify them offline, so they are labeled as copied from upstream.
- The shared SCCACHE_DIR sits under the athena state directory, outside job dirs.
- setup.sh itself may need no change if xtask setup does the install; the user's wording 'setup.sh must install it' is met via the handoff.

## Acceptance criteria

1. [content] athena.toml [capabilities.change.env] keeps CARGO_TARGET_DIR = {job_dir}/target/{side}, and adds RUSTC_WRAPPER pointing at the pinned sccache, a shared SCCACHE_DIR outside any job directory, and CARGO_INCREMENTAL=0; athena.toml is named as changed.
2. [content] athena.toml [tools] gains an sccache pin (version plus per-platform sha256 checksums), shaped like the gitleaks pin, and config.rs parses it.
3. [content] xtask setup installs sccache from the pinned release archive, refuses on checksum mismatch, is idempotent, and is a numbered step like gitleaks. setup.sh and xtask changes are listed in the summary.
4. [content] docs/decisions.md gains a decision amending D46: content-addressed cache keeps isolation, alternatives (shared target dirs, seeded target copy), buy-first call, CARGO_INCREMENTAL=0 rationale.
5. [content] Design text claiming every job builds cold (§10.4, M26, risk table) is updated; any env pass-through needed is added.
6. [content] check_build_dirs still refuses a CARGO_TARGET_DIR outside the job directory; no per-job build directory is shared.
7. [behavior] Run `cargo test` from the repo root: all tests pass, including env.rs, config parsing of athena.toml, and xtask tests (new tests cover the sccache pin and checksum logic).
8. [behavior] Run `cargo clippy --all-targets --locked -- -D warnings` and `cargo fmt --all --check`: both clean.

## Expected deliverables

- athena.toml (modify) (a governance file: allowed because the reading lists it)
- xtask/src/main.rs (modify)
- crates/athena-data/src/config.rs (modify)
- docs/decisions.md (modify)
- docs/design/athena-v0.md (modify)
- crates/athena-agent/src/env.rs (modify)
- setup.sh (modify) (a governance file: allowed because the reading lists it)
- CHANGELOG.md (modify)
- README.md (modify)
- Cargo.toml (modify)
- Cargo.lock (modify)

## Out of scope

- Sharing or persisting CARGO_TARGET_DIR across jobs
- Changing job isolation, sandboxing, or the check_build_dirs rule
- Measuring or tuning cache size and eviction
- Changes to records/ or the owner's files

## User defaults that apply

- buy-first: Adopt good, maintained tools and crates before building; build only for needs they don't meet, and record the call. Here: Adopt sccache rather than building a cache; record the call.
- tested: Every module has passing tests; cargo test passes and clippy is clean. Here: Add tests for the pin parsing and env; cargo test and clippy must pass.
- mechanism: Rules that matter are enforced by mechanism, or explicitly marked prose-only. Here: Pinned checksum enforced in code; build-dir check retained.
- evidence: Claims cite their sources, and numbers are measured or labeled as estimates. Here: Decision cites D46 and any speedup is labeled an estimate unless measured.
- rust: All code is Rust. Non-Rust files appear only where a platform requires them, and each is listed with its reason. Here: Installation logic goes in xtask Rust; setup.sh stays a minimal shim.

## The change

Base 9cc244b5a970 → candidate 136315da49d2. In your working directory, `git diff HEAD~1` shows it.

```text
 CHANGELOG.md                     |   5 ++
 README.md                        |   2 +-
 athena.toml                      |  14 +++
 crates/athena-agent/src/env.rs   |  72 ++++++++++++++-
 crates/athena-data/src/config.rs |  33 ++++++-
 docs/decisions.md                |  30 +++++++
 docs/design/athena-v0.md         |  20 +++--
 docs/enforcement.md              |   2 +-
 xtask/src/main.rs                | 185 +++++++++++++++++++++++++++++++++------
 9 files changed, 321 insertions(+), 42 deletions(-)
```

(The diff was inlined for the verifier; it is elided in this record.)

## Checks Athena ran on this tree

- fmt: `cargo fmt --all --check` passed
- clippy: `cargo clippy --all-targets --locked -- -D warnings` passed
- test: `cargo test --locked` passed (324 tests ran)

## Commands Bash may run

- cargo build | check | test | clippy | fmt | run | metadata | tree
- git status | diff | log | show
- ls (paths inside your working directory)

## Verdict

End with the verdict as your structured output: one criteria entry per acceptance criterion, in order.
