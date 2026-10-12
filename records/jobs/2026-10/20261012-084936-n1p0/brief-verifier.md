# Job 20261012-084936-n1p0: verify "Carry failed check output into retry and referenced-job briefs"

## Request (verbatim)

> retries still can't see why a check failed (20261012-084859-gxq4); carry the failing check's output too

## Reading (accepted by the user)

The reader brief for 'try again on <job>' carries the rejected job's findings and reasons but not why its checks failed. Make `carry` in reading.rs also include each failed check's name, command, exit code and recorded output tail, redacted and size-bounded.

Assumptions:
- 'Retries' means the reader brief for 'try again on <job>' (and referenced jobs), which currently carries only findings and reasons, not the failed CheckRun.output_tail stored in verification.json.
- Only failed checks' output is added; the producer brief is not changed.
- Output is truncated to a modest cap (the stored output_tail is already a tail).

## Acceptance criteria

1. [content] `carry` in crates/athena-core/src/reading.rs adds, for each check in the carried job's final verification with passed == false, an entry naming the check, its command, exit code, and its output_tail, placed in EarlierAttempt.findings after the findings and reasons.
2. [content] Passing checks contribute nothing; a verification with no failed checks yields the same findings as before.
3. [content] The check output goes through the same redactor as the other carried text (redact_all), and is bounded in size so one noisy check cannot bloat the reader brief.
4. [content] Both a retried job (earlier_attempt) and a job referenced by id (referenced_jobs) carry the failed-check output, since both use `carry`; the brief wording in briefs.rs still labels it as data.
5. [behavior] A new unit or lifecycle test builds a rejected job whose verification has a failed check with a distinctive output_tail, and asserts the reader brief for 'try again on <job>' contains that output and the check name, and omits output of a passing check. Run `cargo test`.
6. [behavior] A test shows a secret-shaped string in the check output is redacted in the carried text. Run `cargo test`.
7. [behavior] `cargo test`, `cargo clippy` and `cargo fmt --check` pass.

## Expected deliverables

- crates/athena-core/src/reading.rs (modify)
- crates/athena-context/src/briefs.rs (modify)
- crates/athena-cli/tests/lifecycle.rs (modify)
- CHANGELOG.md (modify)

## Out of scope

- Changing what verification.json records for checks
- Changing the producer or verifier briefs
- Changing retry detection or the 'try again on' syntax

## User defaults that apply

- tested: Every module has passing tests; cargo test passes and clippy is clean. Here: Add tests for carrying failed-check output and for redaction.
- small: Prefer one focused change; size is a design choice, not a wish list. Here: One focused change in carry(), no new structs.
- public: Everything committed is public: no secrets, credentials, or personal data, and transcripts are redacted. Here: Carried check output is redacted like other carried text.
- lazy-requests: Requests are short; recover hidden requirements from the repository before asking. Here: Located the gap in reading.rs carry() from the repository rather than asking.

## The change

Base 5dae6fe56068 → candidate 71d103b975ff. In your working directory, `git diff HEAD~1` shows it.

```text
 CHANGELOG.md                         |   4 ++
 crates/athena-cli/tests/lifecycle.rs |  37 ++++++++++++
 crates/athena-context/src/briefs.rs  |  10 +++-
 crates/athena-core/src/reading.rs    | 106 ++++++++++++++++++++++++++++++++++-
 4 files changed, 154 insertions(+), 3 deletions(-)
```

(The diff was inlined for the verifier; it is elided in this record.)

## Checks Athena ran on this tree

- fmt: `cargo fmt --all --check` passed
- clippy: `cargo clippy --all-targets --locked -- -D warnings` passed
- test: `cargo test --locked` passed (232 tests ran)

## Commands Bash may run

- cargo build | check | test | clippy | fmt | run | metadata | tree
- git status | diff | log | show
- ls (paths inside your working directory)

## Verdict

End with the verdict as your structured output: one criteria entry per acceptance criterion, in order.
