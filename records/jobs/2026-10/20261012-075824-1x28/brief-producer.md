# Job 20261012-075824-1x28: Show the reader any named job's request and findings

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
