# Job 20261010-221301-sakk: Verify landings by records, not commit ancestry or trailers

## Request (verbatim)

> Make Athena work when PRs are squash- or rebase-merged, not just merge commits. I squash PRs here so main reads as one intent per PR, and other projects using Athena will pick their own strategy.

## Reading (accepted by the user)

Make the governed-content invariant and `athena check --history` accept a landing that was squash- or rebase-merged, without relying on Athena-* trailers or on the prediction commit being an ancestor. The job is identified from the records the commit adds, and "prediction before work" is proved from the records themselves (reading and job.json content plus recorded timestamps and invocation order). Merge commits keep working.

Assumptions:
- Trailers are optional hints: the job is identified from the records a commit adds (candidate.json, verification.json, landed outcome.json under records/jobs/<id>/), and a trailer that disagrees with the records is refused.
- Prediction-before-work is proved from record content that survives a squash: the accepted reading file exists in the landed tree, job.json names it, and the reading's recorded time/identity precedes the producer and verifier invocations that the records list. An Athena-Prediction trailer, if present, is not required and ancestry is not used.
- The 'no second landing' rule is enforced by the history walk and by a records-based check of the parent tree (the job already has a landed outcome there), rather than by grepping Athena-Job trailers.
- Rebase-merged landings are handled by the same records-based path; the parent is taken as the first parent of the landing commit on main.
- The existing merge-commit path keeps working and may keep accepting trailers as before when they are valid.
- Only `crates/athena-gate` and the docs change; the landing flow in athena-core/athena-cli keeps emitting trailers, since they are harmless.

## Acceptance criteria

1. [behavior] A new gate test squash-merges a verified landing onto main as one commit whose message has no Athena-* trailers; `cargo test -p athena-gate` shows the history check and invariant report no violations for it.
2. [behavior] A new gate test rebase-merges a verified landing (new commit ids, prediction commit not an ancestor by id); `cargo test -p athena-gate` shows it passes.
3. [behavior] A new test shows a squash commit that changes governed content without the job's valid records is refused with Rule::Invariant; `cargo test -p athena-gate` passes.
4. [behavior] A new test shows a squashed landing whose records show the reading was written after the work (prediction not before the invocations) is refused; `cargo test -p athena-gate` passes.
5. [behavior] A new test shows a second landing of the same job after a squash is refused, and an edited or conflicting trailer on a squash does not make an otherwise valid landing pass or fail wrongly; `cargo test -p athena-gate` passes.
6. [behavior] Existing merge-commit tests still pass: `cargo test`, `cargo clippy --all-targets -- -D warnings`, and `cargo fmt --check` are all clean.
7. [content] docs/design/athena-v0.md §7.3/§7.4 describes how the invariant treats merge, squash and rebase merges, that trailers are not relied on, how prediction-before-work is proved from records, and the alternatives considered, citing the code.
8. [content] The module doc of crates/athena-gate/src/invariant.rs matches the new rules.
9. [content] CHANGELOG.md has an entry, and README.md says merge, squash and rebase merges are all supported.

## Expected deliverables

- crates/athena-gate/src/invariant.rs (modify)
- crates/athena-gate/src/history.rs (modify)
- crates/athena-gate/src/jobs.rs (modify)
- crates/athena-gate/src/lib.rs (modify)
- crates/athena-gate/src/content.rs (modify)
- crates/athena-git/src/lib.rs (modify)
- crates/athena-core/src/land.rs (modify)
- crates/athena-cli/tests/lifecycle.rs (modify)
- docs/design/athena-v0.md (modify)
- CHANGELOG.md (modify)
- README.md (modify)

## Out of scope

- Changing how athena itself creates landing commits or its trailers
- Enforcing a merge strategy or configuring the hosting platform
- Changes to governance files (athena.toml, CI workflows, lint config)
- Changing record schemas unless strictly needed to prove prediction order

## User defaults that apply

- tested: Every module has passing tests; cargo test passes and clippy is clean. Here: New gate tests for squash, rebase, refusals, and double landing; cargo test and clippy clean.
- strangers: Code, commits, docs, and records are written for an outside reader. Here: Design doc, changelog, and README updated for outside readers.
- designed: Significant designs are written down with alternatives and reviewed before they are built. Here: The change to design §7.3/§7.4 records alternatives, including trailer-based and ancestry-based proofs.
- lazy-requests: Requests are short; recover hidden requirements from the repository before asking. Here: Recovered the invariant's rules from invariant.rs and used the user's reply to drop trailer dependence.
- small: Prefer one focused change; size is a design choice, not a wish list. Here: Confined to the gate's landing check plus docs.
- verified: Done means independently verified, and user-facing behavior is verified by exercising it. Here: Behavior criteria are checkable with cargo test, clippy, fmt.
- rust: All code is Rust. Non-Rust files appear only where a platform requires them, and each is listed with its reason. Here: All code and tests are Rust.

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

## Report

End with the producer report as your structured output.
