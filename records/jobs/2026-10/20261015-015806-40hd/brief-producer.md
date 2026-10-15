# Job 20261015-015806-40hd: Put unmet criteria, findings and failed checks into job introspections

## Request (verbatim)

> try again on 20261015-012203-y8dx

## Reading (accepted by the user)

When a rejected or failed job is introspected, event_of_job reads the job's stored verification.json and adds each unmet criterion's text, each blocker/major finding, and failed checks (under a separate heading) to the event's `what`. This retries a job that was rejected for missing wiring tests. Audience: the steward and anyone grouping rejections by real cause.

Assumptions:
- Verification is already loadable from the job's stored records in event_of_job.
- The user prompt is not redacted today; added text is bounded, and its source output_tail was redacted when written.
- Wiring tests through event_of_job are the previous attempt's missing piece.

## Acceptance criteria

1. [content] event_of_job in crates/athena-core/src/introspect.rs reads the job's stored verification and appends each unmet criterion's text and each blocker/major finding (severity, summary, location) to the Event's `what`, with count and character caps; the module docs field list names these additions.
2. [content] Failed checks (passed=false) appear under a separate, labelled 'failed checks' heading with name and a bounded output_tail, never in the unmet-criteria list; SYSTEM_PROMPT tells the model criteria and checks are different. No transcripts, diffs or other jobs' files are added.
3. [content] A job with no verification.json, or with no failures, yields the old `what` with no empty failure section; if the stored result lacks details, the text says so explicitly.
4. [behavior] Tests call event_of_job itself (not only helper functions) on stored job records: (a) one unmet criterion plus a major finding both appear in the event; (b) only a failed test check appears as a failed check with no unmet criterion; (c) a job without verification keeps the old `what`. Run `cargo test -p athena-core introspect`.
5. [behavior] Outcome check using the fake model (crates/athena-cli/tests/introspection.rs style recording of prompts): for a rejected job whose verification.json resembles that of job 20261014-170031-2r9b, the user prompt sent to the model contains the unmet criterion text and the major finding location. Run `cargo test`; `cargo clippy --all-targets -- -D warnings` and `cargo fmt --all --check` are clean.
6. [content] CHANGELOG.md has an entry that accurately describes the mechanism (it does not claim the prompt is redacted unless the code does so), and the SYSTEM_PROMPT literal is wrapped consistently.

## Expected deliverables

- crates/athena-core/src/introspect.rs (modify)
- CHANGELOG.md (modify)
- crates/athena-cli/tests/introspection.rs (modify)
- docs/design/defaults-loop.md (modify)
- docs/design/decision-sets.md (modify)

## Out of scope

- Adding a pre-verification self-check step or repair loop to jobs
- Changing the verifier or the verification.json schema
- Reading transcripts, diffs or other jobs' files in introspection

## User defaults that apply

- tested: Every module has passing tests; cargo test passes and clippy is clean. Here: Tests exercise event_of_job end to end; cargo test and clippy are clean.
- root-cause: Fix the root cause, not the symptom or the one instance that surfaced. The fault is fixed where it originates; when that lies outside the job's scope, name it and propose the fix rather than working around it. The change description names the root cause and why the change addresses it. The root-cause fix is the one focused change. Here: Fix the input the model sees, not the model's wording.
- verified: Done means independently verified, and user-facing behavior is verified by exercising it. Here: A behavior criterion checks the prompt content with the fake model.
- small: Prefer one focused change; size is a design choice, not a wish list. Here: One focused change in introspect.rs with docs and changelog.
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
