# Job 20261012-190155-rd0v: Answer review 2 in the squash-chain landing design

## Request (verbatim)

> answer review 2 of the squash-chain design, then get it built

## Reading (accepted by the user)

As the user directed, this job only answers review 2 in the squash-chain design: it adds a response appendix for the six findings and revises §4.1, §4.6 and §5 to match. It also updates D69 and the changelog. Nothing is built; the build is a later job.

Assumptions:
- Per the user's reply, this job only answers review 2 in the design; the build is a separate later job, so no Rust code, schema, or tests change.
- previous_job is assumed derived by the core as the latest landed job in the base commit's history (absent if none); the design states this as a changeable assumption.
- D69 stays proposed because the chain rule is not yet built.
- Line rewrapping (R2-n2) is applied to the cited decisions.md and v0 design lines only if small.

## Acceptance criteria

1. [content] docs/design/squash-chain-landing.md gains an Appendix B answering each review-2 finding (R2-M1, R2-m1, R2-m2, R2-m3, R2-n1, R2-n2) with a disposition and the section changed; the review file is unchanged.
2. [content] §4.1 defines how the core derives previous_job from the base commit (including after a hand change, a records-only commit, and an unlanded predecessor), states it as an assumption the user may change, and says what is written when none is found.
3. [content] The design resolves R2-m2: one remaining job is judged by the single-job rule (or the head rule is extended, one stated), how a landed predecessor is found at P is specified, and a /1 predecessor is accepted or refused explicitly; §4.6 is made consistent.
4. [content] The design resolves R2-m1 (merge-commit refusal message: either note that the existing merge message is shown or list a change to check that passes landing's reason through, with a test) and notes rules changes in R2-n1.
5. [content] §5 items 1 and 2 name AnyRecord, the schema-name table, the consumers (athena-core run.rs, store.rs, athena-gate jobs.rs), and put RecordSet::load in athena-gate.
6. [content] D69 in docs/decisions.md and the CHANGELOG entry are updated to reflect the answers; D69 stays proposed and no code or tests change; the build is left as a separate later job.
7. [behavior] Run `cargo test` and `cargo fmt --check`: both pass (the repository's wording scan covers the docs).

## Expected deliverables

- docs/design/squash-chain-landing.md (modify)
- docs/decisions.md (modify)
- CHANGELOG.md (modify)
- docs/design/athena-v0.md (modify)
- docs/enforcement.md (modify)

## Out of scope

- Building the chain rule: athena.candidate/2, core writes, gate landing rule, jobs::check and history changes, and code tests (a later job).
- Editing the review file itself.
- Changing any governance file.

## User defaults that apply

- designed: Significant designs are written down with alternatives and reviewed before they are built. Here: The design is revised with alternatives and a response appendix before any build.
- evidence: Claims cite their sources, and numbers are measured or labeled as estimates. Here: Each answer cites the review finding and the code it was checked against.
- small: Prefer one focused change; size is a design choice, not a wish list. Here: Design-only change; the build is split into a later job.
- strangers: Code, commits, docs, and records are written for an outside reader. Here: The appendix is written for an outside reader.

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
