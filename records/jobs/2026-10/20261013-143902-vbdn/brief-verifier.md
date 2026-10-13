# Job 20261013-143902-vbdn: verify "Carry accepted corrections into retries of failed jobs"

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

## The change

Base 458968fd3873 → candidate 321fcfdf2884. In your working directory, `git diff HEAD~1` shows it.

```text
 CHANGELOG.md                        |   5 ++
 crates/athena-context/src/briefs.rs |  21 ++++++
 crates/athena-core/src/reading.rs   | 127 +++++++++++++++++++++++++++++++-----
 3 files changed, 137 insertions(+), 16 deletions(-)
```

(The diff was inlined for the verifier; it is elided in this record.)

## Checks Athena ran on this tree

- fmt: `cargo fmt --all --check` passed
- clippy: `cargo clippy --all-targets --locked -- -D warnings` passed
- test: `cargo test --locked` passed (345 tests ran)

## Commands Bash may run

- cargo build | check | test | clippy | fmt | run | metadata | tree
- git status | diff | log | show
- ls (paths inside your working directory)

## Verdict

End with the verdict as your structured output: one criteria entry per acceptance criterion, in order.
