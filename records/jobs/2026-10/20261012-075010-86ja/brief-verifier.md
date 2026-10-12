# Job 20261012-075010-86ja: verify "Carry the original request when retrying a retry"

## Request (verbatim)

> review 1 is still unanswered: retrying a retry dropped the original request; fix that, then get review 1 answered

## Reading (accepted by the user)

Fix `earlier_attempt` in reading.rs so that retrying a job that was itself a retry carries the original request, found by following the chain with loop protection. Per the user's reply, nothing else is done: no review-1 response and no design revision.

Assumptions:
- Scope is only the earlier_attempt fix in crates/athena-core/src/reading.rs; the review-1 response and design revision are out of scope per the user's reply.
- The user's remark that the 8 decisions are answered needs no action in this job.
- The original request is found by following 'try again on <id>' requests back to the first job that is not a retry.

## Acceptance criteria

1. [behavior] Retrying a retry: A rejected, B ('try again on A') rejected, C asked 'try again on B' gets an Earlier attempt whose request contains A's original request text, and B's findings still appear. New test in crates/athena-cli/tests/lifecycle.rs (or reading.rs unit tests); run `cargo test`.
2. [behavior] A single retry (A not itself a retry) yields the same brief as before; existing retry tests pass. Run `cargo test`.
3. [behavior] Errors unchanged: unknown job, non-rejected job, and bare 'try again on' are still refused with an error naming the job. Run `cargo test`.
4. [behavior] Everything carried, including the original request, passes through the redactor; a test shows it. `cargo test`, `cargo clippy`, `cargo fmt --check` are clean.
5. [content] Following the chain cannot loop forever or panic: a visited-id set or depth bound is used, and a code comment says which.
6. [content] CHANGELOG.md [Unreleased] records the retry fix. No design doc, review response, or governance file is changed.

## Expected deliverables

- crates/athena-core/src/reading.rs (modify)
- crates/athena-cli/tests/lifecycle.rs (modify)
- CHANGELOG.md (modify)
- crates/athena-context/src/briefs.rs (modify)

## Out of scope

- Answering design review 1 and revising the study capability design
- Any code, crate, or config for the study capability
- Governance files

## User defaults that apply

- small: Prefer one focused change; size is a design choice, not a wish list. Here: Only the retry fix, as the user asked.
- tested: Every module has passing tests; cargo test passes and clippy is clean. Here: Add a retry-of-retry test; cargo test and clippy stay clean.
- lazy-requests: Requests are short; recover hidden requirements from the repository before asking. Here: The user's reply settles scope; the remaining details are decided by the producer.
- public: Everything committed is public: no secrets, credentials, or personal data, and transcripts are redacted. Here: Carried text, including the original request, is redacted.

## The change

Base 5dbe05f16013 → candidate e6ad2e5a9b2b. In your working directory, `git diff HEAD~1` shows it.

```text
 CHANGELOG.md                         |  6 +++
 crates/athena-cli/tests/lifecycle.rs | 75 ++++++++++++++++++++++++++++++++++++
 crates/athena-context/src/briefs.rs  | 11 ++++++
 crates/athena-core/src/reading.rs    | 65 +++++++++++++++++++++++++------
 4 files changed, 146 insertions(+), 11 deletions(-)
```

(The diff was inlined for the verifier; it is elided in this record.)

## Checks Athena ran on this tree

- fmt: `cargo fmt --all --check` passed
- clippy: `cargo clippy --all-targets --locked -- -D warnings` passed
- test: `cargo test --locked` passed (223 tests ran)

## Commands Bash may run

- cargo build | check | test | clippy | fmt | run | metadata | tree
- git status | diff | log | show
- ls (paths inside your working directory)

## Verdict

End with the verdict as your structured output: one criteria entry per acceptance criterion, in order.
