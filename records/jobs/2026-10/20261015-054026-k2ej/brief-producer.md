# Job 20261015-054026-k2ej: Make steward items above its limit approvable proposals

## Request (verbatim)

> The steward filed two items above its limit (records/work-items/2026-10/20261015-033454-1xb2.json and -d4re.json) as repeated failures, not proposals, so nothing was queued and approve-work refuses them. Anything above the steward's limit must reach me as something I can approve or reject.

## Reading (accepted by the user)

The steward keeps the kind repeated-failure on an item above its limit. No request is queued for it, and approve-work and the status list only accept kind proposal. The fix is that anything above the limit is recorded as a proposal, so the owner can approve or reject it.

Assumptions:
- The audience is the owner, who runs athena approve-work and athena status.
- Root cause: validate_item in crates/athena-core/src/steward.rs maps the model's repeated-failure kind to WorkItemKind::RepeatedFailure regardless of rung, and find_open and open_proposals accept only Proposal. The fix is in the steward, so every item above the limit is a proposal. Widening approve-work to accept RepeatedFailure items is the rejected alternative.
- The two existing records are under records/, which cannot change, so they are not converted here. The owner needs a new steward run, or a follow-up, to re-file them as proposals.
- The repeated-failure kind stays for items within the limit, where a request is queued.

## Acceptance criteria

1. [behavior] A unit test in crates/athena-core/src/steward.rs passes a repeated-failure item on a rung above the role's limit through validation and decide. The resulting work item has kind Proposal and no request is queued. Run cargo test -p athena-core steward.
2. [behavior] A repeated-failure item at or below the limit still becomes a RepeatedFailure work item with a queued request. The existing test scripted_output_with_a_real_gap_commits_the_work_items_and_a_queued_request still passes (cargo test -p athena-cli --test steward).
3. [behavior] A CLI-level test runs the steward with a scripted repeated-failure item above the limit. The queued item appears in open_proposals (the list athena status shows) and approve_work accepts it instead of refusing. Run cargo test -p athena-cli.
4. [content] The rule that anything above the steward's limit is a proposal, whatever kind the model named, is stated in a code comment or the design doc. If the steward prompt said otherwise, it is corrected.
5. [behavior] cargo fmt --check, cargo clippy and cargo test pass for the workspace.

## Expected deliverables

- crates/athena-core/src/steward.rs (modify)
- crates/athena-cli/tests/steward.rs (modify)
- crates/athena-cli/tests/work_decisions.rs (modify)
- docs/design/roles-and-ladder.md (modify)
- CHANGELOG.md (modify)
- crates/athena-data/src/roles.rs (modify)

## Out of scope

- Editing or migrating records/work-items/2026-10/20261015-033454-1xb2.json and -d4re.json
- Changing the steward's ladder or limits
- Changing how approve-work builds its request

## User defaults that apply

- root-cause: Fix the root cause, not the symptom or the one instance that surfaced. The fault is fixed where it originates; when that lies outside the job's scope, name it and propose the fix rather than working around it. The change description names the root cause and why the change addresses it. The root-cause fix is the one focused change. Here: Fix the kind assignment where the steward's items are validated, not the approve-work refusal for the two instances.
- tested: Every module has passing tests; cargo test passes and clippy is clean. Here: Add unit and CLI tests for the above-limit repeated-failure case; clippy and fmt stay clean.
- verified: Done means independently verified, and user-facing behavior is verified by exercising it. Here: Behavior criteria run cargo tests that exercise the approve path.
- small: Prefer one focused change; size is a design choice, not a wish list. Here: One focused change in the steward with its tests.
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
