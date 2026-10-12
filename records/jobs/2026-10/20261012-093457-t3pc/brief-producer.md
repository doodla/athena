# Job 20261012-093457-t3pc: Let "try again on" accept failed jobs and carry their reason

## Request (verbatim)

> a job that failed admission (20261012-090901-9vwq) can't be tried again and its reason isn't carried; fix both

## Reading (accepted by the user)

`athena ask "try again on <job>"` currently accepts only rejected jobs. I read the request as making it accept failed jobs too (such as one refused at admission) and carrying the recorded outcome reason into the reader's brief as the earlier attempt's finding.

Assumptions:
- 'try again on <job>' (athena ask) is the retry path; `athena go` on closed jobs stays terminal, per the user's reply.
- The failure reason is already stored in the outcome record (Happened::with_reason in run.rs), so the fix is reading it back, not changing what is recorded.
- A failed-at-admission job has a request record but may have no verification record; the outcome reason is the only failure text available.
- Failed jobs that did have verification still carry its findings in addition to the outcome reason.

## Acceptance criteria

1. [content] In crates/athena-core/src/reading.rs, earlier_attempt accepts a job closed as failed as well as rejected; other states (open, accepted, stale, etc.) still bail with a message naming the state and that only rejected or failed jobs can be tried again.
2. [content] carry includes the job's recorded outcome reason (e.g. 'admission refused the change: ...') as a finding, redacted, when the job has no verification record. A failed job with no verification record no longer triggers the 'no verification record' bail.
3. [content] The 'Earlier attempt' section in crates/athena-context/src/briefs.rs says the job failed or was rejected according to its actual state, rather than always 'was rejected'; the new request still takes precedence.
4. [behavior] A new test in crates/athena-cli/tests/lifecycle.rs makes a job fail at admission, then asks 'try again on <job>' and shows the reader brief contains the admission refusal reason and the original request. Run `cargo test -p athena-cli --test lifecycle`.
5. [behavior] Unit tests in reading.rs and briefs.rs cover a failed job being accepted and its reason carried, and a non-retryable state still being refused. Run `cargo test`.
6. [behavior] `cargo fmt --check` and `cargo clippy --all-targets` run clean, and the existing retry tests still pass.
7. [content] CHANGELOG.md and README.md (the 'try again on' paragraph) say that failed jobs can be tried again and that their recorded reason is carried into the reader's brief.

## Expected deliverables

- crates/athena-core/src/reading.rs (modify)
- crates/athena-context/src/briefs.rs (modify)
- crates/athena-cli/tests/lifecycle.rs (modify)
- CHANGELOG.md (modify)
- README.md (modify)
- crates/athena-core/src/run.rs (modify)
- crates/athena-core/src/manage.rs (modify)

## Out of scope

- Making `athena go` retry closed jobs
- Changing what the outcome record stores
- Retrying jobs closed as stale or other states
- Changes to the study capability

## User defaults that apply

- tested: Every module has passing tests; cargo test passes and clippy is clean. Here: Add unit and lifecycle tests for retrying a failed job; cargo test and clippy must pass.
- small: Prefer one focused change; size is a design choice, not a wish list. Here: Limit the change to the retry path (earlier_attempt, carry, brief wording) plus docs.
- strangers: Code, commits, docs, and records are written for an outside reader. Here: Update CHANGELOG and README for an outside reader.
- verified: Done means independently verified, and user-facing behavior is verified by exercising it. Here: Exercise the behavior through the lifecycle test.

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
