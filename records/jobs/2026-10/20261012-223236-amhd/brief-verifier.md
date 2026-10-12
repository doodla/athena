# Job 20261012-223236-amhd: verify "Pin the study instance's repo ref in core and test dispatch calls it"

## Request (verbatim)

> try again on 20261012-221537-kmv5

## Reading (accepted by the user)

When a study job starts, athena-core resolves the repo URL and ref from the study instance's config to a SHA through a Pinner trait. It records the SHA in the Job record and mounts the pinned tree read-only in the agents' working directory. Nothing is hard-coded to firstmate, and a dispatch-level test with a fake pinner proves run.rs calls it.

Assumptions:
- pin.rs is absent at this commit because the earlier attempt was rejected, so it is recreated.
- The study instance config table carries the repo URL and ref; if it lacks them, no pin is made, or the job fails clearly.
- The real pinner shells out to git (ls-remote and a shallow fetch or checkout) rather than adding a git crate.
- Read-only is enforced by file permissions plus agent policy; a hard OS-level mount is not required.

## Acceptance criteria

1. [content] athena-core has a Pinner trait with a real git-based implementation that resolves a repo URL and ref to a full 40-hex SHA and checks out that tree. The core, not an agent, makes the call.
2. [content] The repo URL and ref come from the study instance's config (for example athena.toml). No firstmate URL or 'main' is hard-coded in Rust source; firstmate appears only as config or docs examples.
3. [content] run.rs calls the pinner only for the study capability, before the prediction commit. It stores URL, ref and SHA in a new optional Job record field. The change capability never calls the pinner.
4. [content] The pinned tree is mounted read-only in the working directory of both producer and verifier. Docs say how read-only is enforced.
5. [behavior] A dispatch-level test builds Athena with a counting fake pinner and a scripted fake agent driver on the study capability. It asserts exactly one pinner call, the fake SHA in the Job record, and the mount existing in the agent working directory. Run `cargo test -p athena-core`.
6. [behavior] A dispatch-level test with the real Change capability and the same counting fake pinner asserts zero calls and no pin in the Job record. Run `cargo test`.
7. [behavior] Real-pinner unit tests use a local file:// repo in a temp dir, with no network. They cover resolving the ref to the expected SHA, an error on a missing ref, and the read-only mount. Run `cargo test -p athena-core`.
8. [behavior] `cargo test`, `cargo clippy --all-targets -- -D warnings` and `cargo fmt --check` pass. Removing the pin call in run.rs would fail at least one test.
9. [content] The decision record (alternatives, reversal cost) is in docs/decisions.md and CHANGELOG.md notes the change. The diff has no secrets and no network-dependent tests.

## Expected deliverables

- crates/athena-core/src/pin.rs (create)
- crates/athena-core/src/run.rs (modify)
- crates/athena-core/src/lib.rs (modify)
- crates/athena-core/src/state.rs (modify)
- crates/athena-cli/src/wiring.rs (modify)
- crates/athena-cap-study/src/lib.rs (modify)
- crates/athena-data/src/config.rs (modify)
- crates/athena-core/src/invocation.rs (modify)
- crates/athena-core/src/fake.rs (modify)
- docs/decisions.md (modify)
- CHANGELOG.md (modify)
- athena.toml (modify) (a governance file: allowed because the reading lists it)
- crates/athena-core/Cargo.toml (modify)
- Cargo.lock (modify)

## Out of scope

- Pinning for the change capability
- Network-dependent tests
- Changing study prompts beyond pointing at the mount path
- Hard-coding firstmate

## User defaults that apply

- tested: Every module has passing tests; cargo test passes and clippy is clean. Here: Dispatch-level and real-pinner tests, with clippy and fmt clean.
- rust: All code is Rust. Non-Rust files appear only where a platform requires them, and each is listed with its reason. Here: All code is Rust.
- small: Prefer one focused change; size is a design choice, not a wish list. Here: One focused change around pinning.
- verified: Done means independently verified, and user-facing behavior is verified by exercising it. Here: Behavior criteria are runnable with cargo test.
- strangers: Code, commits, docs, and records are written for an outside reader. Here: Docs and the decision record are written for outside readers.
- public: Everything committed is public: no secrets, credentials, or personal data, and transcripts are redacted. Here: Tests use local file:// repos and no secrets are added.
- mechanism: Rules that matter are enforced by mechanism, or explicitly marked prose-only. Here: A test fails if the run.rs pin call is removed.

## The change

Base 680dff408cfe → candidate 0c3f8f78b52b. In your working directory, `git diff HEAD~1` shows it.

```text
 CHANGELOG.md                                |   5 +
 athena.toml                                 |   5 +
 crates/athena-cap-study/prompts/producer.md |   2 +-
 crates/athena-cap-study/src/lib.rs          |   4 +
 crates/athena-cli/src/wiring.rs             |   3 +
 crates/athena-cli/tests/common/mod.rs       |   2 +
 crates/athena-cli/tests/lifecycle.rs        |  22 ++
 crates/athena-core/src/capability.rs        |   5 +
 crates/athena-core/src/fake.rs              |  55 +++++
 crates/athena-core/src/lib.rs               |   9 +-
 crates/athena-core/src/pin.rs               | 335 ++++++++++++++++++++++++++++
 crates/athena-core/src/run.rs               |  67 +++++-
 crates/athena-core/src/run_pin_tests.rs     | 286 ++++++++++++++++++++++++
 crates/athena-core/src/state.rs             |   3 +-
 crates/athena-data/src/config.rs            |   6 +
 crates/athena-data/src/records.rs           |  15 ++
 docs/decisions.md                           |  38 ++++
 17 files changed, 853 insertions(+), 9 deletions(-)
```

(The diff was inlined for the verifier; it is elided in this record.)

## Checks Athena ran on this tree

- fmt: `cargo fmt --all --check` passed
- clippy: `cargo clippy --all-targets --locked -- -D warnings` passed
- test: `cargo test --locked` passed (302 tests ran)

## Commands Bash may run

- cargo build | check | test | clippy | fmt | run | metadata | tree
- git status | diff | log | show
- ls (paths inside your working directory)

## Verdict

End with the verdict as your structured output: one criteria entry per acceptance criterion, in order.
