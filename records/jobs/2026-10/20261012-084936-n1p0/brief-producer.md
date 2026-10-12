# Job 20261012-084936-n1p0: Carry failed check output into retry and referenced-job briefs

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
