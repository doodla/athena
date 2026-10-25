# Job 20261012-190155-rd0v: verify "Answer review 2 in the squash-chain landing design"

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

## The change

Base b2417f750673 → candidate c15adde3fad4. In your working directory, `git diff HEAD~1` shows it.

```text
 CHANGELOG.md                        |   6 ++
 docs/decisions.md                   |  22 ++++---
 docs/design/athena-v0.md            |   3 +-
 docs/design/squash-chain-landing.md | 111 +++++++++++++++++++++++++++++-------
 docs/enforcement.md                 |   2 +-
 5 files changed, 112 insertions(+), 32 deletions(-)
```

(The diff was inlined for the verifier; it is elided in this record.)

## Checks Athena ran on this tree

- fmt: `cargo fmt --all --check` passed
- clippy: `cargo clippy --all-targets --locked -- -D warnings` passed
- test: `cargo test --locked` passed (267 tests ran)

## Commands Bash may run

- cargo build | check | test | clippy | fmt | run | metadata | tree
- git status | diff | log | show
- ls (paths inside your working directory)

## Verdict

End with the verdict as your structured output: one criteria entry per acceptance criterion, in order.
