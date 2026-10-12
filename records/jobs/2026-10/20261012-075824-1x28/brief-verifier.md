# Job 20261012-075824-1x28: verify "Show the reader any named job's request and findings"

## Request (verbatim)

> when a request names a job id, let the reader see that job's request and findings, not just for 'try again on'

## Reading (accepted by the user)

Today only 'try again on <job>' (rejected jobs only) gives the reader the earlier job's request and findings. I read the request as: any request that names a full job id gets that job's request and findings in the reader brief, while the retry form keeps working.

Assumptions:
- Naming means a full job id (like 20261012-075824-1x28) matched by its format, to avoid false matches on ordinary words; unique suffixes are only resolved in the 'try again on' form.
- Any job state is shown, not only rejected; the retry form keeps its rejected-only rule.
- The number of referenced jobs is capped to bound brief size.
- The brief shows the referenced job's request and findings as quoted data; the new request takes precedence.

## Acceptance criteria

1. [content] A request that contains a full job id anywhere in its text (not only after 'try again on') causes the reader brief to include that job's request and its final verification findings and reasons, redacted like the retry path.
2. [content] Jobs in any state with a request record are shown; a job with no verification gets only its request. An id that does not resolve is ignored, not an error, unless the request uses the 'try again on' form.
3. [behavior] 'try again on <job>' still works: refuses landed or unknown jobs and carries a rejected job's request and findings. Run cargo test -p athena-cli --test lifecycle and cargo test for athena-core and athena-context.
4. [behavior] New tests cover the brief rendering for a referenced job and detection of ids mid-sentence, and a lifecycle test shows a non-retry request naming a job gets its request and findings in the reader brief. Run cargo test, cargo clippy, cargo fmt --check.
5. [content] README.md and CHANGELOG.md describe the behavior.

## Expected deliverables

- crates/athena-core/src/reading.rs (modify)
- crates/athena-context/src/briefs.rs (modify)
- crates/athena-core/src/run.rs (modify)
- crates/athena-cli/tests/lifecycle.rs (modify)
- CHANGELOG.md (modify)
- README.md (modify)

## Out of scope

- Resolving unique id suffixes outside the 'try again on' form
- Changing the producer or verifier briefs
- Changing record formats or governance files

## User defaults that apply

- tested: Every module has passing tests; cargo test passes and clippy is clean. Here: New unit and lifecycle tests for referenced-job detection and brief rendering.
- lazy-requests: Requests are short; recover hidden requirements from the repository before asking. Here: Read the existing 'try again on' implementation to infer the intended generalization.
- small: Prefer one focused change; size is a design choice, not a wish list. Here: Extend the existing earlier-attempt mechanism rather than adding a new subsystem.
- public: Everything committed is public: no secrets, credentials, or personal data, and transcripts are redacted. Here: Carried text goes through the redactor.

## The change

Base 863df7f9603f → candidate 95ff7facd7df. In your working directory, `git diff HEAD~1` shows it.

```text
 CHANGELOG.md                         |   4 ++
 README.md                            |   5 ++
 crates/athena-cli/tests/lifecycle.rs |  59 ++++++++++++++++++
 crates/athena-context/src/briefs.rs  |  54 +++++++++++++++++
 crates/athena-core/src/reading.rs    | 113 +++++++++++++++++++++++++++++------
 5 files changed, 216 insertions(+), 19 deletions(-)
```

(The diff was inlined for the verifier; it is elided in this record.)

## Checks Athena ran on this tree

- fmt: `cargo fmt --all --check` passed
- clippy: `cargo clippy --all-targets --locked -- -D warnings` passed
- test: `cargo test --locked` passed (225 tests ran)

## Commands Bash may run

- cargo build | check | test | clippy | fmt | run | metadata | tree
- git status | diff | log | show
- ls (paths inside your working directory)

## Verdict

End with the verdict as your structured output: one criteria entry per acceptance criterion, in order.
