# Job 20261012-210319-g0a1: verify "Register the study capability in the CLI so Athena can run it"

## Request (verbatim)

> back to the study: make study a capability Athena can run, then start firstmate M1

## Reading (accepted by the user)

Wire the existing athena-cap-study crate into the athena binary so the reader can pick `study` and the core can run it, with the dependency edge documented and tested. Starting the firstmate M1 study itself is not part of this change: the design says containment, the run step, and the Pages design are unbuilt and the study is a separate job.

Assumptions:
- `make study a capability Athena can run` means registering the existing `athena-cap-study` crate in `athena-cli`'s `capabilities()`. The crate and its `Capability` impl already exist but are not registered (crates/athena-cap-study/src/lib.rs header; wiring.rs lists only Change).
- `start firstmate M1` is a separate later job: it needs a pin, containment, the run step and the Pages design, none of which exist (docs/design/study-capability-m1.md §8), and it must make no unreviewed claims about firstmate.
- No `[capabilities.study]` section is added to athena.toml in this change. The study capability then runs with no configured checks or commands, as its own test expects; adding one is a governance change left for a later job.
- The dependency edge cli -> cap-study must be mirrored in docs/architecture.md, because xtask/tests/policy.rs checks the documented edges.

## Acceptance criteria

1. [content] crates/athena-cli/src/wiring.rs `capabilities()` returns both `athena_cap_change::Change` and `athena_cap_study::Study`.
2. [content] crates/athena-cli/Cargo.toml depends on `athena-cap-study.workspace = true`, and Cargo.lock reflects the new edge.
3. [content] docs/architecture.md lists `athena-cap-study` among athena-cli's dependencies and no longer says the crate is 'Not yet registered in the CLI'. The crate doc comment in athena-cap-study/src/lib.rs and the Partly built note in the design doc are updated to match.
4. [content] CHANGELOG.md has an entry saying the study capability is registered, and docs/decisions.md records the call only if it differs from D64/D65.
5. [content] A test in athena-cli asserts that the registered capability ids include `change` and `study`, with no duplicates.
6. [content] No change to athena.toml, defaults.toml, workflows, or other governance files, and nothing about firstmate is claimed or started.
7. [behavior] Run `cargo test` (including xtask policy tests): all pass, including the architecture dependency-edge check.
8. [behavior] Run `cargo clippy --all-targets` and `cargo fmt --check`: both are clean.

## Expected deliverables

- crates/athena-cli/src/wiring.rs (modify)
- crates/athena-cli/Cargo.toml (modify)
- Cargo.lock (modify)
- docs/architecture.md (modify)
- crates/athena-cap-study/src/lib.rs (modify)
- CHANGELOG.md (modify)
- xtask/tests/policy.rs (modify)
- docs/design/study-capability-m1.md (modify)
- docs/decisions.md (modify)
- crates/athena-cli/tests/binary.rs (modify)
- athena.toml (modify) (a governance file: allowed because the reading lists it)

## Out of scope

- Running or reading firstmate, pinning a SHA, or any claim about firstmate
- Containment, the run step, replay, and Pages publishing
- Adding a [capabilities.study] section to athena.toml
- Extending the Capability trait for multi-step jobs
- Milestones M2 to M9

## User defaults that apply

- small: Prefer one focused change; size is a design choice, not a wish list. Here: One focused change: register the capability. The firstmate study is deferred to its own job.
- tested: Every module has passing tests; cargo test passes and clippy is clean. Here: Adds a registration test, and cargo test and clippy must pass.
- mechanism: Rules that matter are enforced by mechanism, or explicitly marked prose-only. Here: The dependency edge is enforced by the existing xtask policy test, so architecture.md must match.
- lazy-requests: Requests are short; recover hidden requirements from the repository before asking. Here: Recovered from the repo that the crate exists and is unregistered, so the request reduces to wiring.
- designed: Significant designs are written down with alternatives and reviewed before they are built. Here: The design (docs/design/study-capability-m1.md) is the source. The Pages design it requires is not built here.
- strangers: Code, commits, docs, and records are written for an outside reader. Here: Docs and comments are updated so an outsider sees the capability is registered.

## The change

Base 142a4f1c010f → candidate 3269f7e3d75d. In your working directory, `git diff HEAD~1` shows it.

```text
 CHANGELOG.md                                |  5 +++++
 Cargo.lock                                  |  1 +
 crates/athena-cap-study/prompts/producer.md | 18 +++++++++---------
 crates/athena-cap-study/src/lib.rs          |  8 ++++----
 crates/athena-cli/Cargo.toml                |  1 +
 crates/athena-cli/src/wiring.rs             | 19 ++++++++++++++++++-
 docs/architecture.md                        |  4 ++--
 docs/design/study-capability-m1.md          |  2 +-
 8 files changed, 41 insertions(+), 17 deletions(-)
```

(The diff was inlined for the verifier; it is elided in this record.)

## Checks Athena ran on this tree

- fmt: `cargo fmt --all --check` passed
- clippy: `cargo clippy --all-targets --locked -- -D warnings` passed
- test: `cargo test --locked` passed (296 tests ran)

## Commands Bash may run

- cargo build | check | test | clippy | fmt | run | metadata | tree
- git status | diff | log | show
- ls (paths inside your working directory)

## Verdict

End with the verdict as your structured output: one criteria entry per acceptance criterion, in order.
