# Job 20261012-090901-9vwq: Build the athena-cap-study crate: records, validation, capability

## Request (verbatim)

> build the study capability

## Reading (accepted by the user)

Build the athena-cap-study crate specified in the reviewed M1 design (D64): typed study records, mechanical validation rules with planted-fault tests, a Capability implementation with prompts, and registration in the CLI. Live runs, containment, and Pages publishing are left for later jobs.

Assumptions:
- 'The study capability' means the athena-cap-study crate of docs/design/study-capability-m1.md (D64), built as a first slice: records, validation rules, Capability impl, prompts, wiring.
- The Pages publishing/replay design and actually running a study are separate later jobs; the run-step containment is built only as far as record types and rules, not executing third-party code.
- Review 2's named changes (R2-m1, R2-m2, R2-m3) are folded into this build since they are small design fixes.
- The design is large; a single job may land only a partial slice and risk verifier rejection.

## Acceptance criteria

1. [content] New crate crates/athena-cap-study exists, is a workspace member in Cargo.toml, and has producer and verifier prompt assets under prompts/.
2. [content] The crate defines typed, schema-tagged records for study.pin/1, source, claim, run, scenario, model, rationale, finding, gap and validation as in design §4, including the R2-m2/R2-m3 finding fields.
3. [content] The §5.4 validation rules are implemented as code, each with a planted-fault test (e.g. missing quote, wrong SHA, uncited claim, inferred rationale shown as stated).
4. [content] A Study type implements the Capability trait with id 'study', a card, and read-only grants; it is registered in crates/athena-cli/src/wiring.rs next to Change.
5. [content] docs/architecture.md, CHANGELOG.md and docs/decisions.md record the crate and the decision to build, and note the Pages-publishing design is still required before M1 is done.
6. [behavior] Run `cargo test` and `cargo clippy --all-targets` from the root: both pass with no warnings; `cargo fmt --check` passes.
7. [behavior] Run `cargo run -- --help` (or the CLI's capability listing): 'study' appears in the capability catalog alongside 'change'.
8. [content] No statement about firstmate, no pinned SHA, and no studied-system run are included.

## Expected deliverables

- crates/athena-cap-study/Cargo.toml (create)
- crates/athena-cap-study/src/lib.rs (create)
- crates/athena-cap-study/src/records.rs (create)
- crates/athena-cap-study/src/validate.rs (create)
- crates/athena-cap-study/prompts/producer.md (create)
- crates/athena-cap-study/prompts/verifier.md (create)
- Cargo.toml (modify)
- Cargo.lock (modify)
- crates/athena-cli/Cargo.toml (modify)
- crates/athena-cli/src/wiring.rs (modify)
- CHANGELOG.md (modify)
- docs/architecture.md (modify)
- docs/decisions.md (modify)
- athena.toml (modify) (a governance file: allowed because the reading lists it)
- docs/design/study-capability-m1.md (modify)

## Out of scope

- Running or modeling firstmate, or any statement about it
- Pinning a SHA or executing a real study
- Pages publishing, explainer page, and replay view design
- Containment sandbox for running third-party code
- Milestones M2–M9

## User defaults that apply

- rust: All code is Rust. Non-Rust files appear only where a platform requires them, and each is listed with its reason. Here: New crate and all code in Rust; prompts are markdown assets as in athena-cap-change.
- tested: Every module has passing tests; cargo test passes and clippy is clean. Here: Each module gets tests including planted-fault tests; cargo test and clippy must pass.
- designed: Significant designs are written down with alternatives and reviewed before they are built. Here: Builds from the reviewed design docs/design/study-capability-m1.md and reviews 1 and 2.
- small: Prefer one focused change; size is a design choice, not a wish list. Here: Limited to the first slice; Pages publishing and live runs excluded.
- lazy-requests: Requests are short; recover hidden requirements from the repository before asking. Here: Recovered scope from D64, the design doc and its §8/§9 rather than asking.
- strangers: Code, commits, docs, and records are written for an outside reader. Here: Docs and rustdoc written for outside readers.

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
