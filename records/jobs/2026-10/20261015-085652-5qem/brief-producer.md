# Job 20261015-085652-5qem: Make drain work in-limit proposals and approved-work jobs

## Request (verbatim)

> athena drain stopped twice without touching the explainer: the audience's implementation-rung items (20261015-064136-gy12, 20261015-085449-1n1k) are listed as needing a person although nothing above the ladder is involved, and the job approve-work created for gy12 (20261015-085558-xebp) is invisible to drain. Make items at or below a role's limit get worked by drain, and make approved work something drain picks up.

## Reading (accepted by the user)

Drain lists audience implementation-rung proposals as "needs a person" and ignores the Proposed job `approve-work` makes, because it only takes steward-channel Queued requests. I read the request as: drain approves open proposals whose rung the raising role may change itself, and works jobs recorded as approved by a work decision. Proposals above the limit still wait for a person. The audience is the operator running `athena drain`.

Assumptions:
- The audience role's proposals are always rung implementation (audience.rs), so drain should compare the item's rung against the limit of the role named in WorkItem.role, using roles::may_change; a missing role definition leaves the item waiting.
- Approving in drain reuses Athena::approve_work, so the request copy and decision record are unchanged; the cost of its reading counts against the budget.
- Approved-work jobs have channel cli and state Proposed, so they are found through Approved work decisions, not the steward channel.
- A newly approved proposal's job is worked in the same drain run, within the existing budget, retry and job-limit rules.

## Acceptance criteria

1. [behavior] A test in crates/athena-cli/tests/drain.rs gives drain an open proposal on the implementation rung from a role whose limit covers it. It shows the proposal is approved and its job is read, run and landed, and that it is not in the report's waiting list. Run `cargo test -p athena-cli --test drain`.
2. [behavior] A test shows an open proposal above its raising role's limit (or from an unknown role) is not approved or worked by drain and still appears as waiting, with the "needs a person" text. Run `cargo test -p athena-cli --test drain`.
3. [behavior] A test shows a job that a work decision records as approved, which is read but not yet run (state proposed), is picked up and run by drain once. A job already closed is not picked up again. Run `cargo test -p athena-cli --test drain`.
4. [content] The module text in drain.rs no longer says drain has no way to approve a proposal. It states that in-limit proposals are approved and approved jobs are worked. Rejecting, and approving above-limit proposals, stay with people.
5. [behavior] `cargo test`, `cargo clippy --all-targets` and `cargo fmt --check` pass.

## Expected deliverables

- crates/athena-cli/src/drain.rs (modify)
- crates/athena-cli/tests/drain.rs (modify)
- crates/athena-core/src/work_decisions.rs (modify)
- crates/athena-core/src/steward.rs (modify)
- CHANGELOG.md (modify)
- crates/athena-cli/src/display.rs (modify)
- crates/athena-cli/src/main.rs (modify)
- docs/design/roles-and-ladder.md (modify)

## Out of scope

- Changing the ladder rules in roles.rs or the top-two-rungs restriction
- Auto-approving proposals above a role's limit
- Addressing the explainer findings themselves
- Changing the audience rating logic

## User defaults that apply

- rust: All code is Rust. Non-Rust files appear only where a platform requires them, and each is listed with its reason. Here: The change is Rust in athena-cli and athena-core.
- tested: Every module has passing tests; cargo test passes and clippy is clean. Here: Stub-driven tests in tests/drain.rs cover both paths; clippy and fmt are clean.
- root-cause: Fix the root cause, not the symptom or the one instance that surfaced. The fault is fixed where it originates; when that lies outside the job's scope, name it and propose the fix rather than working around it. The change description names the root cause and why the change addresses it. The root-cause fix is the one focused change. Here: Both gaps are fixed in how drain builds its queue, not by special-casing the two named items.
- mechanism: Rules that matter are enforced by mechanism, or explicitly marked prose-only. Here: The limit check uses the existing may_change function, not prose.
- small: Prefer one focused change; size is a design choice, not a wish list. Here: One focused change to drain's queue and approval step.
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
