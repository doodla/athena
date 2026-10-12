# Job 20261012-082029-da5v: Fix retry-of-retry dropping the original request

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

The verifier sees only the candidate diff, in a fresh snapshot. It does not see your commands, output, or reasoning. Evidence for a behavior criterion must cite commands the verifier can run itself in that snapshot; your own git status and git diff runs do not count as evidence. Commit hashes made during verification do not exist in the real history, so never pin or cite a commit hash in code, docs, or design-revision text (BRIEF.md included); describe the content instead.

## Report

End with the producer report as your structured output.
