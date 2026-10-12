# Job 20261012-212953-rw6t: verify "Make study selectable; Athena resolves and pins firstmate's SHA at start"

## Request (verbatim)

> try again on 20261012-211640-3x8m; agents have no network, so Athena's own code resolves and records firstmate's SHA when a study starts, not the producer now

## Reading (accepted by the user)

Add the missing [capabilities.study] section so study is selectable. Add a core hook so that Athena's own networked code resolves firstmate's ref to a full SHA and records it when a study starts. The producer has no network and only reads the pinned, read-only source.

Assumptions:
- Reading: study is registered in crates/athena-cli/src/wiring.rs but absent from athena.toml, so it is not selectable. I read this from the repository; the study unit test also asserts the section is absent.
- Athena's own process has network access and runs git for resolution. Agents have none, as the request says.
- The resolved tree is checked out or exported read-only into the job directory, and the SHA is written to records and passed through the study env, e.g. ATHENA_STUDY_SOURCE_SHA.
- The firstmate repository URL is not in the repository files I read, so the producer must take it from the user's answer or a clearly marked config key.
- Only the SHA-resolution and checkout path is in scope; the full run step, replay and publishing stay deferred (D65).

## Acceptance criteria

1. [content] athena.toml gains [capabilities.study] with producer_commands (no rm/mv/chmod wildcard, no network-capable commands such as git fetch, clone or curl), verifier_commands (read-only), checks, and env; it also names firstmate's repository URL and the ref to resolve, and holds no hand-typed SHA. athena.toml is named in this criterion as a governance-file change.
2. [content] A Capability hook with a default no-op body (so `change` is unchanged) lets the core call a capability when a job starts. Study implements it in Athena's Rust code, not in a prompt or producer command: it resolves the configured ref to a full 40-hex SHA, makes the tree available read-only, and records the SHA in the job's records and the producer's environment or brief.
3. [content] The core invokes this hook in the job-start path, so a study job actually resolves and records the SHA. Nothing is left uncalled.
4. [content] The producer prompt says the source is pinned, read-only and present, and that claims must cite the recorded full SHA. It has no other rewording.
5. [content] CHANGELOG.md and docs/decisions.md state that Athena resolves firstmate's SHA at study start, that the producer has no network, and that no SHA is hand-recorded. They do not say a user must add one.
6. [content] The reader's catalog lists `study` because its section is now in athena.toml. The wiring test for registered capabilities still passes. The study test that asserted the capability is absent from athena.toml is updated.
7. [behavior] Run `cargo test --locked`. Offline tests, using an injected runner or a local fixture repository, cover: resolving a ref to a full SHA; rejecting a short, non-hex or empty result; rejecting an unresolvable ref; the hook being a no-op for `change`; and the hook being called at study start.
8. [behavior] Run `cargo clippy --all-targets --locked -- -D warnings` and `cargo fmt --all --check`; both pass.

## Expected deliverables

- athena.toml (modify) (a governance file: allowed because the reading lists it)
- crates/athena-core/src/capability.rs (modify)
- crates/athena-core/src/run.rs (modify)
- crates/athena-cap-study/src/lib.rs (modify)
- crates/athena-cap-study/src/source.rs (create)
- crates/athena-cap-study/prompts/producer.md (modify)
- CHANGELOG.md (modify)
- docs/decisions.md (modify)
- crates/athena-cap-study/Cargo.toml (modify)
- Cargo.lock (modify)
- crates/athena-cap-change/src/lib.rs (modify)
- crates/athena-cli/src/wiring.rs (modify)
- crates/athena-data/src/config.rs (modify)

## Out of scope

- Letting the producer fetch, clone or choose the SHA
- Hand-recording a SHA in athena.toml
- The study run step, replay, containment and publishing (D65)
- Rewording the producer prompt beyond the pinned read-only source and SHA citation
- Changing `change` behavior

## User defaults that apply

- rust: All code is Rust. Non-Rust files appear only where a platform requires them, and each is listed with its reason. Here: The resolver and hook are Rust; the only non-Rust edits are the required TOML and Markdown.
- tested: Every module has passing tests; cargo test passes and clippy is clean. Here: Offline unit tests for the resolver and hook, with clippy and tests passing.
- mechanism: Rules that matter are enforced by mechanism, or explicitly marked prose-only. Here: The SHA is resolved by code and the producer grant has no network commands, rather than by prompt instruction.
- buy-first: Adopt good, maintained tools and crates before building; build only for needs they don't meet, and record the call. Here: Reuse the existing athena-git helpers and the git binary before adding any crate.
- evidence: Claims cite their sources, and numbers are measured or labeled as estimates. Here: CHANGELOG and decision notes cite the earlier rejected attempt's findings.
- small: Prefer one focused change; size is a design choice, not a wish list. Here: One change covers the selectable section and SHA pinning; the run step and the rest stay out.

## The change

Base c2a69078cc16 → candidate 242539a971b1. In your working directory, `git diff HEAD~1` shows it.

```text
 CHANGELOG.md                                |   7 +-
 Cargo.lock                                  |   2 +
 athena.toml                                 |  18 ++
 crates/athena-cap-change/src/lib.rs         |  23 +++
 crates/athena-cap-study/Cargo.toml          |   2 +
 crates/athena-cap-study/prompts/producer.md |   2 +-
 crates/athena-cap-study/src/lib.rs          | 122 +++++++++++-
 crates/athena-cap-study/src/source.rs       | 297 ++++++++++++++++++++++++++++
 crates/athena-cli/src/wiring.rs             |   9 +
 crates/athena-core/src/capability.rs        |  24 +++
 crates/athena-core/src/run.rs               |  23 ++-
 crates/athena-data/src/config.rs            |  13 ++
 docs/decisions.md                           |  26 +++
 13 files changed, 556 insertions(+), 12 deletions(-)
```

(The diff was inlined for the verifier; it is elided in this record.)

## Checks Athena ran on this tree

- fmt: `cargo fmt --all --check` passed
- clippy: `cargo clippy --all-targets --locked -- -D warnings` passed
- test: `cargo test --locked` passed (305 tests ran)

## Commands Bash may run

- cargo build | check | test | clippy | fmt | run | metadata | tree
- git status | diff | log | show
- ls (paths inside your working directory)

## Verdict

End with the verdict as your structured output: one criteria entry per acceptance criterion, in order.
