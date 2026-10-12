# Job 20261012-095109-f1x7: Propose a remedy for the history check failing on squash ee4b0dd

## Request (verbatim)

> main's history check has failed since PR #8 was squash-merged (ee4b0dd: one commit with many job landings); what do you propose?

## Reading (accepted by the user)

I read this as a request for a written proposal on why main's history check fails after the multi-job squash merge ee4b0dd, with options and a recommendation recorded as a decision entry. It does not implement the fix.

Assumptions:
- 'What do you propose?' asks for a written proposal, not an immediate fix, so the deliverable is a decision entry with options and a recommendation.
- The history check fails because the landing check judges a squash as one job's landing, and ee4b0dd carries records of several jobs (read from design §7.3 and landing_tests.rs; the exact failure output was not available to me).
- The full 40-character id of ee4b0dd is not available in the snapshot, so the proposal does not edit athena.toml.
- Recommendation leans toward (b) or (a): exempting is cheap and low reversal-cost for this one commit, while (c) prevents recurrence; the producer should pick on evidence from the code.

## Acceptance criteria

1. [content] docs/decisions.md gains a new, next-numbered decision entry (Decision, Alternatives, Tradeoff, Reversal cost, Provenance) on the failing history check after the squash-merge of PR #8 (ee4b0dd, many job landings in one commit).
2. [content] The entry names the cause with citations to the landing check in crates/athena-gate (and design §7.3), stating that a squash is judged as one landing of one job, so a commit carrying several jobs' records fails; claims cite files read or commands run.
3. [content] The entry compares at least three options: (a) exempt ee4b0dd via [history] exempt (D60/D61, full 40-character id), (b) teach the landing check to accept a commit that lands several jobs when each job's records are complete, (c) stop squashing multi-job PRs (merge commit or rebase merge) and rewrite nothing; it recommends one and says why.
4. [content] No file other than docs/decisions.md (and optionally CHANGELOG.md) changes; athena.toml, workflows and code are untouched, and the entry says implementation is a follow-up job awaiting the user's choice.
5. [behavior] Run `cargo test` and `cargo clippy` from the repository root: both still pass (docs-only change; any test that checks decision numbering or docs still holds). `cargo fmt --check` is clean.

## Expected deliverables

- docs/decisions.md (modify)
- CHANGELOG.md (modify)
- docs/design/athena-v0.md (modify)
- athena.toml (modify) (a governance file: allowed because the reading lists it)
- crates/athena-gate/src/landing_tests.rs (modify)

## Out of scope

- Rewriting main's history or force-pushing.
- Changing the landing-check code or tests.
- Editing athena.toml to add an exemption.
- Changing CI workflows.

## User defaults that apply

- designed: Significant designs are written down with alternatives and reviewed before they are built. Here: The remedy touches the gate, so it is written down with alternatives before anything is built.
- evidence: Claims cite their sources, and numbers are measured or labeled as estimates. Here: Cause and options cite the gate code, design §7.3, and D60/D61.
- small: Prefer one focused change; size is a design choice, not a wish list. Here: One docs change; implementation is left to a follow-up job.
- lazy-requests: Requests are short; recover hidden requirements from the repository before asking. Here: Recovered the history exemption mechanism and squash handling from the repository instead of asking.
- strangers: Code, commits, docs, and records are written for an outside reader. Here: Entry written for an outside reader, in the existing decision format.

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
