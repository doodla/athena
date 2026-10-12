# Job 20261012-191052-dpax: Add athena.candidate/2 and have the core write chain fields

## Request (verbatim)

> build the squash-chain landing

## Reading (accepted by the user)

Per the user's split, this job only adds the athena.candidate/2 record (base_fingerprint, optional previous_job) and has the core write those fields. The gate's chain landing rule, jobs/history changes and the "implemented" doc updates are left for the next job.

Assumptions:
- Design doc docs/design/squash-chain-landing.md option B (D69) defines the field names and derivation of previous_job.
- Consumers that match AnyRecord::Candidate need only minimal changes to accept /2.
- The gate rule job (invariant.rs chain check, jobs.rs/history chain recording, enforcement.md rows, D69 implemented) is a separate later job.

## Acceptance criteria

1. [content] athena-data records.rs defines athena.candidate/2 with base_fingerprint and optional previous_job, registered in AnyRecord and the schema-name table; athena.candidate/1 still parses.
2. [content] docs/records.md documents athena.candidate/2 and its two new fields.
3. [content] athena-core run.rs writes /2 candidates with base_fingerprint computed from the base commit and previous_job set to the latest landed job in the base's history (absent if none).
4. [content] store.rs and every other AnyRecord::Candidate consumer (scoring, athena-score, athena-git snapshot, gate jobs.rs) compile and handle /2 without changing gate landing, jobs or history rules.
5. [content] Tests cover /2 round-trip, /1 still parsing, and the core writing base_fingerprint and previous_job (including previous_job absent when no job has landed).
6. [content] Gate invariant.rs landing rule is unchanged (single-job refusal stays). The design doc is not marked implemented; at most a CHANGELOG/doc note says the record and core half is done and the gate rule is next. No governance file or records/ path changes.
7. [behavior] Run cargo test, cargo clippy --all-targets, and cargo fmt --check from the repo root; all pass.

## Expected deliverables

- crates/athena-data/src/records.rs (modify)
- crates/athena-core/src/run.rs (modify)
- crates/athena-core/src/store.rs (modify)
- docs/records.md (modify)
- CHANGELOG.md (modify)
- docs/design/squash-chain-landing.md (modify)
- crates/athena-gate/src/jobs.rs (modify)
- crates/athena-gate/src/landing_tests.rs (modify)
- crates/athena-git/src/snapshot.rs (modify)
- crates/athena-core/src/scoring.rs (modify)
- crates/athena-score/src/lib.rs (modify)
- crates/athena-cli/tests/lifecycle.rs (modify)

## Out of scope

- Gate landing chain check in invariant.rs and its refusal messages
- jobs.rs check and history walk accepting multi-job chains
- Gate chain tests (fork, unlinked, broken link, etc.)
- Marking D69, design-doc status, enforcement.md or athena-v0.md §7.3 as implemented
- Governance files and records/

## User defaults that apply

- small: Prefer one focused change; size is a design choice, not a wish list. Here: Split per user: only the record schema and core writing in this job.
- tested: Every module has passing tests; cargo test passes and clippy is clean. Here: Tests for /2 parsing and core writing; cargo test and clippy must pass.
- rust: All code is Rust. Non-Rust files appear only where a platform requires them, and each is listed with its reason. Here: All changes are Rust plus Markdown docs.
- strangers: Code, commits, docs, and records are written for an outside reader. Here: Docs and changelog written for outside readers.

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
