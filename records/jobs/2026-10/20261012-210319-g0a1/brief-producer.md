# Job 20261012-210319-g0a1: Register the study capability in the CLI so Athena can run it

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

The verifier sees only the candidate diff, in a fresh snapshot. It does not see your commands, output, or reasoning. Evidence for a behavior criterion must cite commands the verifier can run itself in that snapshot; your own git status and git diff runs do not count as evidence. Commit hashes made during verification do not exist in the real history, so never pin or cite a commit hash in code, docs, or design-revision text (BRIEF.md included); describe the content instead.

## Report

End with the producer report as your structured output.
