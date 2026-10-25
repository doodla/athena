# Job 20261013-143902-vbdn: Carry accepted corrections into retries of failed jobs

## Request (verbatim)

> Retries of a job that failed (not rejected) lose its accepted corrections: hrdp, retrying failed 9e8t, dropped the opt-in-merge correction. Carry them for failed jobs too.

## Reading (accepted by the user)

When the user says "try again on <failed job>", the retry's reader brief must carry that job's accepted corrections, as it does for rejected jobs. I found no failed-only branch that drops corrections in carry/earlier_attempt/reader_brief. So the producer must reproduce the loss with a failed-job test and fix wherever it appears; likely suspects are retry chains and failed jobs with no verification record.

Assumptions:
- The loss happens in the brief-building path (reading.rs earlier_attempt/carry, or briefs.rs reader_brief), not in record storage.
- A failed job's corrections can be lost when the failed job is itself a retry (its corrections, or its predecessors', are not carried through the chain), or when it has no verification record.
- Only the reader brief for the retry needs the corrections; the producer and verifier briefs are unchanged.
- Carried corrections stay redacted and quoted as data, as today.

## Acceptance criteria

1. [content] A unit test in crates/athena-core/src/reading.rs builds a failed job (Failed outcome) that has a final reading and accepted corrections, and asserts earlier_attempt carries them in order and redacted.
2. [content] A test covers a failed job that has corrections but no verification record, and one that is itself a retry, and asserts its corrections reach the EarlierAttempt.
3. [content] The reader brief for a failed earlier attempt lists the corrections under the existing corrections heading, with a briefs.rs test asserting this.
4. [behavior] Run `cargo test` from the repository root: all tests pass, including the new failed-job correction tests.
5. [behavior] Run `cargo clippy --all-targets` and `cargo fmt --check`: both clean.

## Expected deliverables

- crates/athena-core/src/reading.rs (modify)
- crates/athena-context/src/briefs.rs (modify)
- crates/athena-cli/tests/lifecycle.rs (modify)
- CHANGELOG.md (modify)
- crates/athena-core/src/store.rs (modify)

## Out of scope

- Changing how corrections are recorded or scored.
- Changing which job states can be retried.
- Any change to governance, CI, or configuration files.
- Editing records/.

## User defaults that apply

- tested: Every module has passing tests; cargo test passes and clippy is clean. Here: Add failing-first tests for the failed-job case; cargo test and clippy must pass.
- small: Prefer one focused change; size is a design choice, not a wish list. Here: Limit the change to the carry path for failed-job retries.
- lazy-requests: Requests are short; recover hidden requirements from the repository before asking. Here: Recovered the retry flow from reading.rs and briefs.rs instead of asking.
- public: Everything committed is public: no secrets, credentials, or personal data, and transcripts are redacted. Here: Test fixtures use synthetic text and redacted tokens only.
- verified: Done means independently verified, and user-facing behavior is verified by exercising it. Here: Behavior criteria are checked with cargo test, clippy and fmt --check.

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
