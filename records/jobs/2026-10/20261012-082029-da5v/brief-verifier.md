# Job 20261012-082029-da5v: verify "Fix retry-of-retry dropping the original request"

## Request (verbatim)

> try again on 20261012-075010-86ja

## Reading (accepted by the user)

When the job being retried was itself "try again on <job>", the reader brief must show the original request (followed through the chain, redacted) plus the retried job's own findings. Only the retry fix, as the user said; the review-1 answer is a separate job. Includes the three earlier review notes: CHANGELOG heading, a bare-refusal test, distinct A/B verdicts.

Assumptions:
- The original request is found by following the retried job's 'try again on <job>' request back to the first non-retry request, with a cycle/depth guard.
- The brief still shows the retried job's own retry phrase only through its findings; the original request replaces it as the carried request.
- The review-1 answer in docs/design is out of scope, per the user's reply.

## Acceptance criteria

1. [behavior] A lifecycle test named retrying_a_retry_carries_the_original_request_redacted passes: A rejected, B ('try again on A') rejected, then 'try again on B'; the third reader brief contains A's original request (redacted) and B's findings. Run: cargo test --locked -p athena-cli --test lifecycle retrying_a_retry
2. [behavior] In that test A and B have different verdict findings (distinct canaries), and the assertions require B's canary present and A's absent. Run: cargo test --locked -p athena-cli --test lifecycle retrying_a_retry
3. [behavior] Existing try-again tests still pass, including landed/unknown refusal and plain retry. Run: cargo test --locked -p athena-cli --test lifecycle try_again
4. [behavior] A test covers bare 'try again on' being refused with the 'say which job' message. Run: cargo test --locked -p athena-core reading
5. [behavior] Reader brief rendering tests still pass. Run: cargo test --locked -p athena-context reader_brief
6. [behavior] Formatting and lints are clean. Run: cargo fmt --check and cargo clippy --locked --all-targets -- -D warnings
7. [content] CHANGELOG.md [Unreleased] has a '### Fixed' entry for the retry-of-retry fix placed after '### Added', leaving the existing review-1 bullet under Added.
8. [content] docs/design/study-capability-m1.md and the review file are not changed.

## Expected deliverables

- crates/athena-core/src/reading.rs (modify)
- crates/athena-cli/tests/lifecycle.rs (modify)
- CHANGELOG.md (modify)
- crates/athena-context/src/briefs.rs (modify)

## Out of scope

- Answering review 1 in docs/design/study-capability-m1.md
- Changing governance files or records/

## User defaults that apply

- tested: Every module has passing tests; cargo test passes and clippy is clean. Here: A new lifecycle test for retry-of-retry and a unit test for the bare refusal.
- small: Prefer one focused change; size is a design choice, not a wish list. Here: Only the retry fix; the design-doc review answer is excluded.
- verified: Done means independently verified, and user-facing behavior is verified by exercising it. Here: Behavior criteria name the cargo commands the verifier can run.
- strangers: Code, commits, docs, and records are written for an outside reader. Here: CHANGELOG entry under a Fixed section placed after Added.

## The change

Base be9310972d94 → candidate b93c74a3bf9c. In your working directory, `git diff HEAD~1` shows it.

```text
 CHANGELOG.md                         |  7 +++
 crates/athena-cli/tests/lifecycle.rs | 98 ++++++++++++++++++++++++++++++++++++
 crates/athena-core/src/reading.rs    | 94 ++++++++++++++++++++++++++++------
 3 files changed, 185 insertions(+), 14 deletions(-)
```

(The diff was inlined for the verifier; it is elided in this record.)

## Checks Athena ran on this tree

- fmt: `cargo fmt --all --check` passed
- clippy: `cargo clippy --all-targets --locked -- -D warnings` passed
- test: `cargo test --locked` passed (229 tests ran)

## Commands Bash may run

- cargo build | check | test | clippy | fmt | run | metadata | tree
- git status | diff | log | show
- ls (paths inside your working directory)

## Verdict

End with the verdict as your structured output: one criteria entry per acceptance criterion, in order.
