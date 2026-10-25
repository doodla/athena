# Job 20261015-005240-hc0x: Add approve/reject by id for steward work-item proposals

## Request (verbatim)

> I can't approve a steward proposal: asking Athena to do work item records/work-items/2026-10/20261015-003903-khz6.json gave a blind reading, because readers can't see records/. Give me a way to approve or reject a proposal by its id, so the work it describes reaches the reader in full.

## Reading (accepted by the user)

Add `athena approve-work <id>` and `athena reject-work <id> --why`, which decide a pending steward work item (kind proposal) by its id, mirroring accept-proposal/reject-proposal. Approving submits an ordinary job whose request text inlines the work item's title, rung, source and the content of each cited record, because readers can't see records/. The audience is the user who approves proposals. The decision is appended as a new record, since work items are never edited.

Assumptions:
- 'Steward proposal' means a WorkItem of kind Proposal: a change above the steward's ladder limit, queued only as a work item. It is not the defaults proposal that accept-proposal already handles.
- A WorkItem stores no request text (ProposedChange.request is dropped in decide), so the request for the approved job is built from the item's title, rung, source and evidence.
- The approved request must carry the cited content itself, such as the cited jobs' requests and outcomes and the source's text. A bare path like records/work-items/... is exactly what produced the blind reading.
- Work items are never edited, so approval or rejection is a new appended record that links the work-item id to the job or to the reason. A new record schema is acceptable, with its path, validation and round-trip test following the existing record pattern.
- Approving goes through the usual `ask` reading flow, like accept_proposal. Only open items can be decided, and an item already decided is refused.
- The CLI surface is two new subcommands beside AcceptProposal and RejectProposal. Their exact names are the producer's call.

## Acceptance criteria

1. [content] Reading the request built for an approved work item shows the item's title, rung, source and evidence, plus the content of each cited record inlined as text. The request contains no bare reference to a records/ path as the only description of the work.
2. [content] A new appended record type (schema, canonical path, validation) holds a work-item decision: approved with a job id, or rejected with a non-empty reason. A work item that has a decision is no longer open, and the store loads these records.
3. [content] athena-cli has new subcommands to approve and to reject a work item by id. They appear in the CLI help, and `athena status` or an equivalent output lists open work-item proposals with their ids so the user can find one.
4. [behavior] Run `cargo test`. New tests pass: approving a seeded proposal-kind work item queues a job whose request text contains the inlined content of its cited evidence. Rejecting records the reason. A second decision, an unknown id, and an empty reason are each refused.
5. [behavior] Run `cargo clippy --all-targets` and `cargo fmt --check`. Both are clean.
6. [content] docs and CHANGELOG.md describe the new commands and say the decision is recorded as an append-only record.

## Expected deliverables

- crates/athena-core/src/steward.rs (modify)
- crates/athena-data/src/records.rs (modify)
- crates/athena-core/src/store.rs (modify)
- crates/athena-cli/src/main.rs (modify)
- crates/athena-core/src/status.rs (modify)
- crates/athena-cli/tests/steward.rs (modify)
- crates/athena-cli/tests/work_decisions.rs (create)
- crates/athena-core/src/lib.rs (modify)
- crates/athena-core/src/work_decisions.rs (create)
- crates/athena-cli/src/display.rs (modify)
- CHANGELOG.md (modify)
- README.md (modify)
- docs/design/roles-and-ladder.md (modify)
- crates/athena-gate/src/records.rs (modify)

## Out of scope

- Changing the defaults-proposal commands (accept-proposal, reject-proposal, check-proposal).
- Giving readers access to records/ in general; only the approved request carries the needed content.
- Changing the steward's model, prompts, or what it queues.
- Editing existing work-item records or backfilling decisions for old items.
- Changes to athena.toml, defaults.toml, or workflows.

## User defaults that apply

- rust: All code is Rust. Non-Rust files appear only where a platform requires them, and each is listed with its reason. Here: All new code is Rust in the existing crates.
- tested: Every module has passing tests; cargo test passes and clippy is clean. Here: New record type and both commands get passing tests; clippy is clean.
- strangers: Code, commits, docs, and records are written for an outside reader. Here: Docs and CHANGELOG explain the commands for an outside reader.
- root-cause: Fix the root cause, not the symptom or the one instance that surfaced. The fault is fixed where it originates; when that lies outside the job's scope, name it and propose the fix rather than working around it. The change description names the root cause and why the change addresses it. The root-cause fix is the one focused change. Here: The fix is that the request carries the work item's content. It does not point the user at a path or give readers special access.
- small: Prefer one focused change; size is a design choice, not a wish list. Here: One focused change that mirrors the existing proposal flow.
- verified: Done means independently verified, and user-facing behavior is verified by exercising it. Here: Behavior is checked by cargo test on the CLI path.
- lazy-requests: Requests are short; recover hidden requirements from the repository before asking. Here: The work-item vs defaults-proposal meaning and the missing request text were recovered from steward.rs and proposals.rs.
- changelog: (not in the defaults model) Here: BRIEF.md asks for tagged releases and a changelog, so the change records itself in CHANGELOG.md.

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

The verifier sees only the candidate diff, in a fresh snapshot. It does not see your commands, output, or reasoning. Evidence for a behavior criterion must cite commands the verifier can run itself in that snapshot; your own git status and git diff runs do not count as evidence. Commit hashes made during verification do not exist in the real history, so never pin or cite a commit hash made during verification in code, docs, or design-revision text (BRIEF.md included); describe the content instead. The one exception is the pinned instance: when the brief has a "Pinned source" section, cite its url, ref and SHA as given.

## Report

End with the producer report as your structured output.
