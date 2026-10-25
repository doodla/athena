# Job 20261013-101627-mnfs: verify "Diagnose zero sccache hits and fix the cache or revert it"

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

## The change

Base 947e48af3482 → candidate 18c4e1189a23. In your working directory, `git diff HEAD~1` shows it.

```text
 CHANGELOG.md                     |  5 +++++
 README.md                        | 10 +++++-----
 athena.toml                      | 10 ----------
 crates/athena-data/src/config.rs | 25 +++++++++++--------------
 docs/decisions.md                | 23 +++++++++++++++++++++++
 docs/design/roadmap.md           |  5 +++--
 6 files changed, 47 insertions(+), 31 deletions(-)
```

(The diff was inlined for the verifier; it is elided in this record.)

## Checks Athena ran on this tree

- fmt: `cargo fmt --all --check` passed
- clippy: `cargo clippy --all-targets --locked -- -D warnings` passed
- test: `cargo test --locked` passed (327 tests ran)

## Commands Bash may run

- cargo build | check | test | clippy | fmt | run | metadata | tree
- git status | diff | log | show
- ls (paths inside your working directory)

## Verdict

End with the verdict as your structured output: one criteria entry per acceptance criterion, in order.
