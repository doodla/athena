# Job 20261012-192220-vs0z: verify "Implement the chain-aware squash landing rule in the gate"

## Request (verbatim)

> now the gate rule for squash-chain landing

## Reading (accepted by the user)

Read as: build the gate half of design D69 (docs/design/squash-chain-landing.md §4.2, §5 items 3–5). The `athena.candidate/2` records and the core's writing of them are already in the repo. `landing` in athena-gate should therefore accept a squash that closes several jobs when they form a valid fingerprint chain, and still refuse every broken variant.

Assumptions:
- 'The gate rule' means the check in crates/athena-gate (invariant.rs landing, jobs.rs, history.rs), not the records or core work. Grep found athena.candidate/2, base_fingerprint and previous_job already in athena-data and athena-core.
- The design in docs/design/squash-chain-landing.md §4.2 and §4.3 is followed as written, including its refusal messages and the refusal of chains on multi-parent commits and chains that touch .gitattributes.
- Single-job landings stay byte-for-byte as strict as today.
- No [history] exempt entry is added or removed, and no governance files change.
- Doc updates (enforcement.md rows, D69 status, v0 §7.3 wording, CHANGELOG) are included as the design's §5 item 5 says.

## Acceptance criteria

1. [behavior] Run `cargo test -p athena-gate`. A hand-built squash that adds the records of two or more jobs linked by previous_job and base_fingerprint, with the last job's fingerprint equal to the squashed tree, passes both invariant::check and jobs::check.
2. [behavior] Run `cargo test -p athena-gate`. Tests show each of these is refused with the message in design §4.3: a /1 candidate in a chain, no head or a wrong head base, a fork, an unlinked or missing middle job, a broken fingerprint link, a last job that does not match the tree, a chain that changes a .gitattributes file.
3. [behavior] Run `cargo test -p athena-gate`. A chain on a commit with two parents is refused. A partly landed chain is judged correctly: one job left uses the single-job rule, and two or more left with a landed predecessor, including a /1 predecessor, are accepted.
4. [behavior] Run `cargo test -p athena-gate`. Chains with repeated fingerprints (a job that reverts an earlier one, a job that changes no governed content) land.
5. [behavior] Run `cargo test -p athena-gate`. Each per-job check (reading, a prediction written after the work, verdict, candidate commits, verifier invocation, usage, outcome choice, earlier landing) refuses the commit when it fails on a middle job. After a chain lands, repeating one of its jobs is refused by the history walk.
6. [behavior] Run `cargo test -p athena-gate`. A single-job squash and every existing landing test behave as before, and `cargo clippy --all-targets` and `cargo fmt --check` are clean.
7. [content] The gate's `landing_job` returns the set of jobs, `jobs::check` accepts every job of a chain, and the history state records every job of a chain.
8. [content] docs/enforcement.md no longer says that squashing a chain is refused and names the new tests. D69 is marked accepted or changed, design §7.3 and the CHANGELOG are updated, and no governance file or records/ path is touched.

## Expected deliverables

- crates/athena-gate/src/invariant.rs (modify)
- crates/athena-gate/src/landing_tests.rs (modify)
- crates/athena-gate/src/jobs.rs (modify)
- crates/athena-gate/src/history.rs (modify)
- docs/enforcement.md (modify)
- docs/decisions.md (modify)
- CHANGELOG.md (modify)
- docs/design/squash-chain-landing.md (modify)
- docs/design/athena-v0.md (modify)
- docs/architecture.md (modify)
- crates/athena-gate/src/lib.rs (modify)
- crates/athena-gate/src/records.rs (modify)
- docs/records.md (modify)

## Out of scope

- Changing athena-data schemas or athena-core's derivation of previous_job; these already exist.
- Passing landing's refusal reason through the merge-commit path in check.
- Adding [history] exempt entries or changing athena.toml, workflows or other governance files.
- Stopping squash-merging of pull requests (option c of D66).
- Signing records or defending against forged records.
- Anything under records/.

## User defaults that apply

- tested: Every module has passing tests; cargo test passes and clippy is clean. Here: Add the design's §5 item 4 tests in landing_tests.rs, jobs.rs and history.rs, starting with a hand-built two-job chain. cargo test and clippy must pass.
- designed: Significant designs are written down with alternatives and reviewed before they are built. Here: The design and its two reviews already exist, so this implements them and does not redesign.
- mechanism: Rules that matter are enforced by mechanism, or explicitly marked prose-only. Here: The rule is enforced by the gate itself, and enforcement.md rows are updated to name the tests.
- lazy-requests: Requests are short; recover hidden requirements from the repository before asking. Here: The missing detail was recovered from D69 and the design doc.
- small: Prefer one focused change; size is a design choice, not a wish list. Here: Limit the change to the gate crate, its tests and the docs that describe it.
- strangers: Code, commits, docs, and records are written for an outside reader. Here: Refusal messages and docs are written for an outside reader.

## The change

Base a390b2a6c93f → candidate fb93769a10cc. In your working directory, `git diff HEAD~1` shows it.

```text
 CHANGELOG.md                            |   9 +
 crates/athena-gate/src/history.rs       |  35 ++-
 crates/athena-gate/src/invariant.rs     | 283 ++++++++++++++---
 crates/athena-gate/src/jobs.rs          |  53 +++-
 crates/athena-gate/src/landing_tests.rs | 517 +++++++++++++++++++++++++++++++-
 docs/architecture.md                    |   6 +-
 docs/decisions.md                       |  13 +-
 docs/design/athena-v0.md                |  17 +-
 docs/design/squash-chain-landing.md     |  17 +-
 docs/enforcement.md                     |  10 +-
 docs/records.md                         |   6 +-
 11 files changed, 900 insertions(+), 66 deletions(-)
```

(The diff was inlined for the verifier; it is elided in this record.)

## Checks Athena ran on this tree

- fmt: `cargo fmt --all --check` passed
- clippy: `cargo clippy --all-targets --locked -- -D warnings` passed
- test: `cargo test --locked` passed (289 tests ran)

## Commands Bash may run

- cargo build | check | test | clippy | fmt | run | metadata | tree
- git status | diff | log | show
- ls (paths inside your working directory)

## Verdict

End with the verdict as your structured output: one criteria entry per acceptance criterion, in order.
