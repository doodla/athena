# Job 20261013-101627-mnfs: Diagnose zero sccache hits and fix the cache or revert it

## Request (verbatim)

> Next on the roadmap (RM-0): sccache has 0 hits over 4 jobs (644+ misses; test check still ~165-190s). Find out why and make builds warm, or revert it if it can't help.

## Reading (accepted by the user)

Find why sccache, wired in athena.toml for the change and study capabilities (D80), gets no hits. The suspects are the wrapper not applied or not installed, per-job paths, or the sandbox. Fix the wiring so a second build hits, or remove the sccache wiring if it can't help, and record the outcome.

Assumptions:
- The 0 hits and 644+ misses come from the user's reports, which I did not measure; I could not run anything.
- The producer can only run cargo build/check/test/clippy/fmt/run/metadata/tree/doc, so it cannot call sccache --show-stats directly. It may need to infer the cause from code, such as how env.rs expands {athena_home} and how checks and agents receive the env, or by running a cargo build through a small xtask or test.
- Likely causes to check: the wrapper binary is missing or not installed because setup --ci skips it, the sccache server is blocked from starting or from using SCCACHE_DIR by the sandbox, absolute-path differences in per-job snapshots and target dirs change keys, the env is not applied to the check runs, or HOME and CARGO_HOME differences change registry source paths.
- If the cause cannot be fixed within the permitted commands, the fallback is to revert, as the user allowed.

## Acceptance criteria

1. [content] A recorded diagnosis (new decision in docs/decisions.md) names the measured cause of the 0 sccache hits, citing the commands run and their output (sccache --show-stats before and after), or states that the cause could not be reproduced.
2. [content] Either athena.toml is fixed so that both [capabilities.change.env] and [capabilities.study.env] apply a working sccache setup, or the RUSTC_WRAPPER, SCCACHE_DIR and CARGO_INCREMENTAL lines and the [tools.sccache] pin are removed from athena.toml, with the decision recorded (revert fallback). athena.toml is named in this criterion as the governance file changed.
3. [behavior] Run `cargo test` from the repository root: all tests pass, including any new or updated test in crates/athena-data/src/config.rs or crates/athena-agent/src/env.rs that pins the chosen wiring (or its absence after a revert).
4. [behavior] Run `cargo clippy --all-targets --locked -- -D warnings` and `cargo fmt --all --check`: both are clean.
5. [content] CHANGELOG.md has an entry, and docs/design/roadmap.md RM-0 item 1 is updated to state the outcome (hit count recorded, or cache reverted) without claiming unmeasured numbers.
6. [content] README.md and setup.sh text about sccache stay consistent with whether the pin remains.

## Expected deliverables

- athena.toml (modify) (a governance file: allowed because the reading lists it)
- docs/decisions.md (modify)
- CHANGELOG.md (modify)
- docs/design/roadmap.md (modify)
- crates/athena-data/src/config.rs (modify)
- crates/athena-agent/src/env.rs (modify)
- README.md (modify)
- xtask/src/main.rs (modify)
- setup.sh (modify) (a governance file: allowed because the reading lists it)
- docs/enforcement.md (modify)

## Out of scope

- Concurrent jobs, native pull-request landing and timings reporting (the other RM-0 items).
- Changing per-job target directories (D46) beyond what the cache diagnosis requires.
- Changing CI workflows or adding sccache to CI.
- The firstmate study (RM-6).

## User defaults that apply

- lazy-requests: Requests are short; recover hidden requirements from the repository before asking. Here: RM-0 item 1 in docs/design/roadmap.md supplies the hidden requirement: diagnose, fix, and record hits and misses.
- evidence: Claims cite their sources, and numbers are measured or labeled as estimates. Here: The diagnosis must cite what was run and read. The user's figures are labeled as reported, not measured.
- mechanism: Rules that matter are enforced by mechanism, or explicitly marked prose-only. Here: A unit test pins the sccache wiring in athena.toml, or its absence after a revert.
- tested: Every module has passing tests; cargo test passes and clippy is clean. Here: cargo test and clippy must pass.
- small: Prefer one focused change; size is a design choice, not a wish list. Here: One focused change, covering only the build cache. Concurrent jobs, PR landing and timings stay out.
- strangers: Code, commits, docs, and records are written for an outside reader. Here: The decision record and changelog are written for an outside reader.

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
