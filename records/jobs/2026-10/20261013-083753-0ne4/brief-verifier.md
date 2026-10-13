# Job 20261013-083753-0ne4: verify "Carry corrections, scope and discard notice into retry readings"

## Request (verbatim)

> Retries must keep the corrections made to the earlier reading and the original request's scope, and know the rejected attempt's changes were discarded (m4yj kept the findings but dropped the strategy view and roadmap).

## Reading (accepted by the user)

When a user says "try again on <job>", the reader brief should also carry the rejected job's corrected reading (its latest reading with the user's corrections applied) and the original request's scope. It should also say plainly that the rejected attempt's changes were discarded, so the retry does not assume they exist or drop what the earlier reading kept (the m4yj case, where findings were kept but the strategy view and roadmap were dropped).

Assumptions:
- Today carry() in crates/athena-core/src/reading.rs gives the reader only the rejected job's request, verification findings and failed checks. It omits that job's readings and the user's corrections to them.
- The fix extends EarlierAttempt (crates/athena-context/src/briefs.rs) with the final reading's summary, criteria and deliverables plus the corrections. reader_brief renders them and states that the attempt's changes were discarded and nothing from it is in the repository.
- Where the retried job was itself a retry, the original request is already followed back (original_request). The corrections from the whole chain should carry too, bounded and redacted like the other carried text.
- The 'm4yj' job is only a motivating example; no job-specific code is needed.
- The producer brief and the Athena-level scope handling are out of scope unless the reading code already feeds them from EarlierAttempt. I found no other users of earlier_attempt beyond run.rs:348, which builds it.

## Acceptance criteria

1. [content] EarlierAttempt in crates/athena-context/src/briefs.rs gains fields for the earlier job's final reading (summary and acceptance criteria) and the user's corrections to it.
2. [content] carry() in crates/athena-core/src/reading.rs fills those fields from the retried job's readings and corrections, passing the text through the redactor and bounding its size.
3. [content] reader_brief renders the carried corrected reading and corrections under the Earlier attempt section. It states that the rejected attempt's changes were discarded and are not in the repository, and that the original request's scope still applies and the corrections must be kept.
4. [content] For a retry of a retry, the original request and the corrections from earlier jobs in the chain are carried.
5. [behavior] Unit tests in briefs.rs and reading.rs assert the brief text contains the corrections, the original scope and the discard notice, and is unchanged when there is no earlier attempt. Run `cargo test` and it passes.
6. [behavior] `cargo clippy` reports no warnings and `cargo fmt --check` passes.
7. [behavior] The existing lifecycle test try_again_on_a_rejected_job_carries_its_request_and_findings_to_the_reader still passes, and is extended to check corrections are carried. Run `cargo test -p athena-cli`.

## Expected deliverables

- crates/athena-context/src/briefs.rs (modify)
- crates/athena-core/src/reading.rs (modify)
- crates/athena-cli/tests/lifecycle.rs (modify)
- CHANGELOG.md (modify)
- crates/athena-core/src/run.rs (modify)
- docs (modify)

## Out of scope

- Changing how the producer or verifier briefs are built, beyond what EarlierAttempt feeds.
- Changing the 'try again on' syntax or which job states can be retried.
- Restoring or re-applying the discarded attempt's file changes.
- Governance files: athena.toml, defaults.toml, CI, toolchain, lint config.

## User defaults that apply

- rust: All code is Rust. Non-Rust files appear only where a platform requires them, and each is listed with its reason. Here: The change is Rust only.
- tested: Every module has passing tests; cargo test passes and clippy is clean. Here: Unit tests are added for the brief rendering and the carry; cargo test and clippy must pass.
- small: Prefer one focused change; size is a design choice, not a wish list. Here: One focused change to the retry carry path and the brief renderer.
- lazy-requests: Requests are short; recover hidden requirements from the repository before asking. Here: The intent was recovered from the retry code in reading.rs and briefs.rs, not by asking.
- public: Everything committed is public: no secrets, credentials, or personal data, and transcripts are redacted. Here: Carried text goes through the existing redactor.
- verified: Done means independently verified, and user-facing behavior is verified by exercising it. Here: The verifier runs cargo test, clippy and fmt --check.

## The change

Base 6072d0474dd3 → candidate 008d29f582dd. In your working directory, `git diff HEAD~1` shows it.

```text
 CHANGELOG.md                         |   6 ++
 crates/athena-cli/tests/lifecycle.rs |  28 +++++
 crates/athena-context/src/briefs.rs  |  71 +++++++++++-
 crates/athena-core/src/reading.rs    | 203 ++++++++++++++++++++++++++++++++---
 4 files changed, 293 insertions(+), 15 deletions(-)
```

(The diff was inlined for the verifier; it is elided in this record.)

## Checks Athena ran on this tree

- fmt: `cargo fmt --all --check` passed
- clippy: `cargo clippy --all-targets --locked -- -D warnings` passed
- test: `cargo test --locked` passed (324 tests ran)

## Commands Bash may run

- cargo build | check | test | clippy | fmt | run | metadata | tree
- git status | diff | log | show
- ls (paths inside your working directory)

## Verdict

End with the verdict as your structured output: one criteria entry per acceptance criterion, in order.
