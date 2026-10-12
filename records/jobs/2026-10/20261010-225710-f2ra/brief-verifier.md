# Job 20261010-225710-f2ra: verify "Carry a rejected job's request and findings into a retry reading"

## Request (verbatim)

> Let me retry a rejected job by its id: 'try again on <job>' should carry that job's request and verifier findings to the reader, which today cannot see them.

## Reading (accepted by the user)

When the request is "try again on <job>", Athena resolves that rejected job and gives the reader its request text and final verifier findings in a new "Earlier attempt" brief section. The new job's own request stays as typed.

Assumptions:
- The form is matched case-insensitively at the start of the request: 'try again on <job-id>'.
- Only rejected jobs can be retried; landed, unknown, or in-flight jobs are refused with an error naming the job and its state.
- The earlier findings are those of the job's final verification record.
- Carried text is redacted before it reaches the brief.

## Acceptance criteria

1. [content] ReaderInputs in crates/athena-context/src/briefs.rs gains an optional earlier-attempt field (job id, request text, findings). reader_brief renders it as its own quoted section only when present; output without it is unchanged.
2. [content] A function in athena-core detects 'try again on <job>', resolves the job, and builds the attempt from that job's request record and final verification findings. It refuses a job that is not rejected, or that lacks a request or verification, with an error naming the job and its state.
3. [content] read_for and first_reading in crates/athena-core/src/reading.rs pass the attempt to reader_brief, and the carried text goes through the redactor.
4. [content] README.md and CHANGELOG.md mention the 'try again on <job>' form.
5. [behavior] Run `cargo test`: a unit test in briefs.rs shows the brief contains the earlier request and findings when set and lacks the section when unset.
6. [behavior] Run `cargo test`: a lifecycle test with the fake driver rejects a job, then asks 'try again on <id>' and shows the reader brief includes that job's request and findings. Another test shows a landed or unknown job is refused.
7. [behavior] Run `cargo clippy --all-targets` and `cargo fmt --check`: both are clean.

## Expected deliverables

- crates/athena-context/src/briefs.rs (modify)
- crates/athena-core/src/reading.rs (modify)
- README.md (modify)
- CHANGELOG.md (modify)
- crates/athena-core/src/lib.rs (modify)
- crates/athena-core/tests/lifecycle.rs (modify)

## Out of scope

- Changing how the verifier or producer work on retries
- Automatically retrying without the user asking
- Editing governance files or records/
- Retrying landed or in-flight jobs

## User defaults that apply

- tested: Every module has passing tests; cargo test passes and clippy is clean. Here: Unit and lifecycle tests cover the new section and the refusal cases.
- small: Prefer one focused change; size is a design choice, not a wish list. Here: One focused change across the brief, the resolver, and the docs.
- public: Everything committed is public: no secrets, credentials, or personal data, and transcripts are redacted. Here: Carried request and findings text goes through the redactor.
- strangers: Code, commits, docs, and records are written for an outside reader. Here: README and CHANGELOG document the new form.
- verified: Done means independently verified, and user-facing behavior is verified by exercising it. Here: Behavior criteria use cargo test, clippy, and fmt --check.

## The change

Base 26a0cfde5566 → candidate bb667a81dfcd. In your working directory, `git diff HEAD~1` shows it.

```text
 CHANGELOG.md                         |  6 +++
 README.md                            |  5 ++
 crates/athena-cli/tests/lifecycle.rs | 98 +++++++++++++++++++++++++++++++++++-
 crates/athena-context/src/briefs.rs  | 66 ++++++++++++++++++++++--
 crates/athena-core/src/reading.rs    | 92 +++++++++++++++++++++++++++++++--
 crates/athena-core/src/run.rs        |  2 +
 6 files changed, 260 insertions(+), 9 deletions(-)
```

(The diff was inlined for the verifier; it is elided in this record.)

## Checks Athena ran on this tree

- fmt: `cargo fmt --all --check` passed
- clippy: `cargo clippy --all-targets --locked -- -D warnings` passed
- test: `cargo test --locked` passed (221 tests ran)

## Commands Bash may run

- cargo build | check | test | clippy | fmt | run | metadata | tree
- git status | diff | log | show
- ls (paths inside your working directory)

## Verdict

End with the verdict as your structured output: one criteria entry per acceptance criterion, in order.
