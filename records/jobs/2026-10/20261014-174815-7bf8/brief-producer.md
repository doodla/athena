# Job 20261014-174815-7bf8: Add role definitions, a steward role, and a requirements ladder

## Request (verbatim)

> Build what the BRIEF.md addendum '2026-10-14: A loop that steers Athena, roles, and a ladder of requirements' asks for, starting small: roles as recorded definitions (A9, A10), and a first role that runs every 5 tasks in parallel with other work, looks at everything Athena has done, and queues work items, including one for anything it could not see. Record only what would otherwise be lost; pull the rest from where it lives. Use the ladder to decide what it may change itself.

## Reading (accepted by the user)

Add recorded role definitions and a first steward role that runs in parallel every 5 finished tasks. It reads Athena's state in place and queues work items, including one for each blind spot. A requirements ladder (owner goals, brief requirements, design decisions, defaults and conventions, implementation) lets it change only the lower rungs itself and propose the upper ones with evidence.

Assumptions:
- Cadence counts finished tasks (landed or rejected) since the steward's last run, kept as a persisted counter.
- The steward writes proposals and work items only; it does not edit the owner's goals or BRIEF.md.
- Data it cannot read (for example records/ or history absent from its view) becomes a blind-spot work item.
- Rungs from design decisions down are the ones the steward may change itself, but this first version only applies changes on the lowest two (defaults/conventions, implementation); upper rungs are proposed with evidence.
- Athena's checks and the independent verifier still gate any change the steward applies.

## Acceptance criteria

1. [content] athena-data has a role definition record (name, goal, prompt, cadence in finished tasks, highest rung it may change itself) and one steward definition with a short stable prompt.
2. [content] A work item record names its source by reference (job id, path or commit) and carries evidence by reference, not copied data. It has a field for the requirement rung it concerns, and a kind for a blind spot.
3. [content] A Ladder enum in athena-data orders five rungs: OwnerGoals, BriefRequirements, DesignDecisions, DefaultsConventions, Implementation. A function decides whether a role may change a rung itself (only DesignDecisions and below, or whatever the role's limit is, never the top two) or must queue a proposal.
4. [behavior] Run cargo test: with the fake agent, finishing the 5th task starts the steward without blocking the main flow, and the 4th does not.
5. [behavior] Run cargo test: when a data source is unreadable, the steward queues a work item that names the blind spot and its source.
6. [behavior] Run cargo test: a proposed change on a rung above the role's limit is queued with evidence and not applied. A change on a lower rung is applied.
7. [behavior] Run cargo clippy and cargo fmt --check, then cargo test: all clean and passing.
8. [content] docs/design/requirements-map.md moves A9-1 and A10-1 to covered, naming the code. CHANGELOG.md has an entry. A short design note gives the alternatives and the ladder.

## Expected deliverables

- crates/athena-data/src/records.rs (modify)
- crates/athena-data/src/roles.rs (create)
- crates/athena-data/src/lib.rs (modify)
- crates/athena-core/src/steward.rs (create)
- crates/athena-core/src/lib.rs (modify)
- crates/athena-core/src/run.rs (modify)
- crates/athena-cli/src/wiring.rs (modify)
- crates/athena-core/src/fake.rs (modify)
- docs/design/requirements-map.md (modify)
- CHANGELOG.md (modify)
- docs/design/roles-and-ladder.md (create)
- docs/records.md (modify)
- Cargo.lock (modify)

## Out of scope

- Further roles beyond the steward
- Role cost and yield tracking, reshaping and retirement (rest of A10-1 and A9-2)
- Changing owner goals, brief requirements or athena.toml
- Copying Athena's state into new records

## User defaults that apply

- rust: All code is Rust. Non-Rust files appear only where a platform requires them, and each is listed with its reason. Here: All new code is Rust in the existing crates.
- tested: Every module has passing tests; cargo test passes and clippy is clean. Here: New modules ship with tests; cargo test and clippy pass.
- small: Prefer one focused change; size is a design choice, not a wish list. Here: Start with one role and one ladder; no other roles.
- strangers: Code, commits, docs, and records are written for an outside reader. Here: The design note and changelog are written for an outside reader.
- lazy-requests: Requests are short; recover hidden requirements from the repository before asking. Here: Details recovered from BRIEF.md and the requirements map, with the user's reply fixing the ladder.
- mechanism: Rules that matter are enforced by mechanism, or explicitly marked prose-only. Here: The ladder is enforced by a function with tests, not prose.
- verified: Done means independently verified, and user-facing behavior is verified by exercising it. Here: Behavior is checked through cargo test with the fake agent.
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
