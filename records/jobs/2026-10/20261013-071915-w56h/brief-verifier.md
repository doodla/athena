# Job 20261013-071915-w56h: verify "Skip sccache install in setup --ci to speed up CI"

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

## The change

Base 182cfd2027a6 → candidate 553d1791a72a. In your working directory, `git diff HEAD~1` shows it.

```text
 CHANGELOG.md      |  5 +++++
 README.md         |  3 ++-
 docs/decisions.md | 14 ++++++++++++++
 xtask/src/main.rs | 19 ++++++++++++++++++-
 4 files changed, 39 insertions(+), 2 deletions(-)
```

(The diff was inlined for the verifier; it is elided in this record.)

## Checks Athena ran on this tree

- fmt: `cargo fmt --all --check` passed
- clippy: `cargo clippy --all-targets --locked -- -D warnings` passed
- test: `cargo test --locked` passed (322 tests ran)

## Commands Bash may run

- cargo build | check | test | clippy | fmt | run | metadata | tree
- git status | diff | log | show
- ls (paths inside your working directory)

## Verdict

End with the verdict as your structured output: one criteria entry per acceptance criterion, in order.
