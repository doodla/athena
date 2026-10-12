# Job 20261012-201841-1r97: Design native PR landing, avoiding the boundary-scan banned words

## Request (verbatim)

> try again on 20261012-201349-pf6d

## Reading (accepted by the user)

Retry the rejected design doc on how Athena lands its work through PRs natively (2026-10-12 addenda: "Athena does what the operator should not have to", brownfield/stealth). The earlier doc failed only the M19 policy test because it used the banned word "operator" about 20 times. This version must pass cargo test.

Assumptions:
- The earlier design content was sound; the only recorded failure was the boundary scan (M19), so the fix is wording plus the same design.
- The design is a document only: no code is built, per 'designed' (design before build).
- Brownfield/stealth addendum applies: PR landing must work without committing Athena records into a third-party repo's branch.
- The banned list (operator, sandbox, lease, handoff, machine paths) is in xtask/tests/policy.rs; it is not edited.

## Acceptance criteria

1. [content] docs/design/native-pr-landing.md exists and covers the problem, requirements from the 2026-10-12 addenda (native PRs, brownfield/stealth), at least two alternatives with a recommendation, and the flow from verified job to PR to merge.
2. [content] The doc addresses the existing prose-only rule that Athena does not push and published history is never rewritten, and says what changes.
3. [content] The doc says how the records, trailers and checks (M6, M7, M23) behave when landing through a PR, and how stealth mode avoids adding Athena files to a third-party repo.
4. [content] No tracked Athena file uses a word the boundary scan bans (the word 'operator', etc.).
5. [behavior] Run `cargo test --locked`; all tests pass, including athena_refers_to_no_one_who_runs_it_or_where in xtask/tests/policy.rs.
6. [behavior] Run `cargo fmt --check` and `cargo clippy --locked`; both are clean (no Rust changes expected).

## Expected deliverables

- docs/design/native-pr-landing.md (create)
- docs/decisions.md (modify)
- docs/enforcement.md (modify)
- CHANGELOG.md (modify)

## Out of scope

- Implementing PR landing in code.
- Editing the boundary scan or its word list.
- Changing BRIEF.md, records/, or governance files.
- Designing the other 2026-10-12 addenda (taste probes, tracker, brief contract).

## User defaults that apply

- designed: Significant designs are written down with alternatives and reviewed before they are built. Here: Write the design with alternatives considered, at docs/design/native-pr-landing.md.
- strangers: Code, commits, docs, and records are written for an outside reader. Here: Written for an outside reader; refers to 'the user' or 'a person' rather than banned terms.
- evidence: Claims cite their sources, and numbers are measured or labeled as estimates. Here: Cite existing docs (squash-chain-landing, design v0 section on pushes, enforcement map M-rows) and the brief addenda.
- mechanism: Rules that matter are enforced by mechanism, or explicitly marked prose-only. Here: State which rules would be enforced by mechanism versus prose-only.
- small: Prefer one focused change; size is a design choice, not a wish list. Here: One design doc, plus at most a decisions-log entry.

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
