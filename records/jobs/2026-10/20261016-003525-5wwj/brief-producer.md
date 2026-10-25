# Job 20261016-003525-5wwj: Make athena drain work user-queued (ask --later) requests too

## Request (verbatim)

> athena drain works only steward and audience items, so requests I queue with 'athena ask --later' (for example 20261015-215658-6ejp, 20261015-215701-cqn8, 20261015-215703-px13, 20261015-225907-kb92) are never worked. Drain should work queued requests from me too.

## Reading (accepted by the user)

Drain's queue (steward::queued_work) filters to channel "steward", while `ask --later` records channel "cli", so user-deferred requests are never worked. Make queued_work include both channels, oldest first, update its docs and tests. Audience: the user who queues requests and expects drain to work them.

Assumptions:
- Requests queued by `athena ask --later` have channel "cli" (reading.rs queue); that is the only user channel to add.
- Drain's existing budget, max_jobs and retry limits keep applying to user requests.
- Mixed order is by received_at then job id, as now.
- The named jobs are examples; this change does not work or close them.

## Acceptance criteria

1. [behavior] A unit test in steward.rs shows queued_work returns unread requests from both the steward channel and the cli channel (as `ask --later` writes), oldest first, and still excludes jobs that are no longer Queued. Run `cargo test -p athena-core queued_work`.
2. [behavior] A drain test with a cli-channel queued request in the records shows drain's QueueView lists it as an item and works it. Run `cargo test -p athena-cli drain`.
3. [content] The doc comments and text that say drain takes only the steward channel and never user-deferred requests (steward.rs REQUEST_CHANNEL, drain.rs, docs/decisions.md entry near line 2206) are corrected, and a decision entry/CHANGELOG line records the change and why.
4. [behavior] `cargo test` passes, `cargo clippy --all-targets` is clean, and `cargo fmt --check` passes.
5. [content] The fix is in the shared queue selection, not a special case for the named job ids; ordering and retry rules are unchanged.

## Expected deliverables

- crates/athena-core/src/steward.rs (modify)
- crates/athena-cli/src/drain.rs (modify)
- crates/athena-cli/tests/drain.rs (modify)
- docs/decisions.md (modify)
- CHANGELOG.md (modify)
- README.md (modify)
- docs/design/roles-and-ladder.md (modify)

## Out of scope

- Working the four named jobs themselves
- Changing drain's budget, retry or rating rules
- Changing how steward proposals are approved

## User defaults that apply

- tested: Every module has passing tests; cargo test passes and clippy is clean. Here: Tests for queued_work and drain with cli-channel requests; cargo test and clippy clean.
- root-cause: Fix the root cause, not the symptom or the one instance that surfaced. The fault is fixed where it originates; when that lies outside the job's scope, name it and propose the fix rather than working around it. The change description names the root cause and why the change addresses it. The root-cause fix is the one focused change. Here: Fix the channel filter in queued_work, not individual jobs.
- behavior-evidence: Acceptance criteria must check the requested behavior happening, not that types, files or wiring exist. A protection or denial test first shows the action succeeds without the protection, as the same user and in the same environment the real component runs as (root), then shows the action is refused. It fails, not skips or passes silently, if the environment cannot enforce the protection; a file mode that root bypasses is not proof. Worked examples are in docs/design/defaults-loop.md section 3.7. Here: Criteria run tests showing a cli request is worked.
- small: Prefer one focused change; size is a design choice, not a wish list. Here: One focused change to queue selection.
- strangers: Code, commits, docs, and records are written for an outside reader. Here: Doc comments and decision/changelog updated.
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
