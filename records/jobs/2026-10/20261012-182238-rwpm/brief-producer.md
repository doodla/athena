# Job 20261012-182238-rwpm: Design squash-merge landing for a chain of verified jobs

## Request (verbatim)

> adopters choose their own merge method: design how a squash of a chain of verified jobs lands cleanly, instead of requiring one job per commit

## Reading (accepted by the user)

Write a design (doc plus decision entry, no code) for how the landing/history gate can accept a squash merge of a chain of sequentially verified jobs, so adopters needn't restrict to one job per commit. It builds on D66–D68 and design §7.3, and recommends one option from the alternatives.

Assumptions:
- 'Design' means a written design with alternatives and a decision entry (the 'designed' default), not an implementation of the gate change.
- The goal is that adopters can squash-merge a PR holding a chain of verified jobs and the history/landing check passes without per-commit exemptions.
- The chain is sequential: each job is based on the previous job's landed or candidate state.
- Existing decisions D60-D68 stay in force; the design proposes and does not supersede them until accepted.

## Acceptance criteria

1. [content] A new design document under docs/design/ (e.g. squash-chain-landing.md) states the problem: a squash of a chain of verified jobs yields one commit adding several jobs' closing records, which `landing` in crates/athena-gate/src/invariant.rs refuses (more than one closing job; whole-tree fingerprint mismatch), citing D66 and design §7.3.
2. [content] The design lists at least three alternatives (e.g. per-job patch composition proof, chain-aware landing rule where only the final job's fingerprint must equal the landed tree and earlier jobs must be ancestors by recorded base, restricting merge methods, exemption) with tradeoffs, and recommends one with reasons.
3. [content] The recommended design specifies what records link a chain (e.g. each job's base tree/fingerprint equals the prior job's verified candidate fingerprint), the exact check the gate would perform on a squash, how it fails (refusal messages), and how it keeps the 'nothing reaches main unverified' and prediction-precedes-outcome (M23) properties.
4. [content] docs/decisions.md gains a new entry (next number after D68) in the existing format (Decision, Alternatives, Tradeoff, Reversal, Provenance) recording the design as proposed, with claims labeled as read from code or inferred, and no implementation claimed.
5. [content] The design names the follow-up implementation work (gate change in invariant.rs, tests in landing_tests.rs, update of design §7.3 and docs/enforcement.md) and states that this change modifies no Rust code, athena.toml, workflows, or other governance files.
6. [content] CHANGELOG.md has a line noting the design; no records/ or governance files change.
7. [behavior] Run `cargo test` and `cargo fmt --check` and `cargo clippy`: all still pass, and `git diff --stat` shows only files under docs/ and CHANGELOG.md.

## Expected deliverables

- docs/design/squash-chain-landing.md (create)
- docs/decisions.md (modify)
- CHANGELOG.md (modify)
- docs/design/athena-v0.md (modify)
- docs/enforcement.md (modify)
- docs/architecture.md (modify)

## Out of scope

- Implementing the gate change in crates/athena-gate
- Changing athena.toml, workflows, or the [history] exempt list
- Rewriting main or splitting commit ee4b0dd
- Configuring repository merge-method settings on the host

## User defaults that apply

- designed: Significant designs are written down with alternatives and reviewed before they are built. Here: Write the design with alternatives before any gate code is built; implementation is a follow-up.
- evidence: Claims cite their sources, and numbers are measured or labeled as estimates. Here: Cite invariant.rs lines, design §7.3, D66-D68; label unrun claims as inferred.
- small: Prefer one focused change; size is a design choice, not a wish list. Here: One design doc plus one decision entry and a changelog line; no code.
- mechanism: Rules that matter are enforced by mechanism, or explicitly marked prose-only. Here: Design states how the rule would be enforced by the gate rather than by prose.
- strangers: Code, commits, docs, and records are written for an outside reader. Here: Doc written for an outside reader with the problem restated.

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
