# Job 20261013-060728-kgp3: Add pinned sccache for warm builds while keeping per-job target dirs

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
