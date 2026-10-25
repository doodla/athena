# Job 20261013-085204-nym7: Carry earlier reading, corrections and discard notice into retries

## Request (verbatim)

> Retry 20261013-083753-0ne4.

## Reading (accepted by the user)

When a request retries a rejected or failed job, the reader's "Earlier attempt" section should also show that job's final reading with the user's corrections and answers, and the original request's scope. It should state that the rejected attempt's changes were discarded and the retry starts from the base. This redoes 0ne4, whose verifier found the README not updated and a behavior criterion with no cited command.

Assumptions:
- The earlier changes from job 0ne4 were discarded, so the work starts from the current base, which has no corrections or discard notice in EarlierAttempt.
- The retry request is 'Retry <id>', which the existing 'try again on' form does not match; it is carried as a Referenced job. The change targets the carried content, not new retry phrasing.
- The 'strategy view and roadmap' finding in 0ne4's request refers to a previous attempt's dropped scope; it is treated as part of the original request that must be carried, not as a deliverable here.

## Acceptance criteria

1. [content] EarlierAttempt in crates/athena-context/src/briefs.rs gains fields for the retried job's final reading (summary, criteria, deliverables) and the corrections or answers made to it; earlier_attempt() in crates/athena-core/src/reading.rs fills them from the job's records, redacted like the other carried text.
2. [content] reader_brief renders, inside the Earlier attempt section, the earlier reading, the corrections and answers (quoted as data), and a sentence saying the rejected attempt's changes were discarded and are not in the repository the reader sees.
3. [content] When the retried job was itself a retry, the carried request is still the original request (existing behavior kept), so the original scope is preserved across chained retries.
4. [content] A failed job with no reading or no corrections renders without empty headings or errors.
5. [content] README.md's retry paragraph (around lines 47-51) says the reader also sees the earlier reading, the corrections, and that the attempt's changes were discarded; CHANGELOG.md has an entry.
6. [content] New unit tests in briefs.rs and reading.rs cover: reading and corrections carried, discard notice present, chained retry keeps the original request, and a job with no corrections.
7. [behavior] Run `cargo test` from the repository root; all tests pass, including the new ones. The verifier's evidence must cite this command.
8. [behavior] Run `cargo clippy --all-targets` and `cargo fmt --check`; both are clean. The verifier's evidence must cite these commands.

## Expected deliverables

- crates/athena-context/src/briefs.rs (modify)
- crates/athena-core/src/reading.rs (modify)
- README.md (modify)
- CHANGELOG.md (modify)
- crates/athena-core/src/run.rs (modify)
- docs/decisions.md (modify)

## Out of scope

- Changing how jobs are landed, stale, or how rejected changes are stored
- Changing the 'try again on' syntax or the Referenced jobs section
- Governance files such as athena.toml, defaults.toml, CI, and lint configuration

## User defaults that apply

- tested: Every module has passing tests; cargo test passes and clippy is clean. Here: New unit tests cover the new fields and rendering; cargo test and clippy must pass.
- verified: Done means independently verified, and user-facing behavior is verified by exercising it. Here: Behavior criteria name the exact commands the verifier must run and cite.
- small: Prefer one focused change; size is a design choice, not a wish list. Here: One focused change to the brief and its builder, plus docs.
- strangers: Code, commits, docs, and records are written for an outside reader. Here: README and CHANGELOG explain what a retry carries.
- public: Everything committed is public: no secrets, credentials, or personal data, and transcripts are redacted. Here: All carried text goes through the redactor.
- rust: All code is Rust. Non-Rust files appear only where a platform requires them, and each is listed with its reason. Here: Only Rust and Markdown files change.

## Words you must not write

Athena's boundary scan fails any tracked file that names who runs Athena or the machine it runs on. Do not write these words in any file you touch: code, comments, docs, tests, prompts, changelog or design text. The scan matches whole words, case-insensitively, so capital letters do not help; singular, plural and hyphenated forms are all listed. Reword instead.

Words: `operator`, `operators`, `opctl`, `ccr`, `sandbox`, `sandboxes`, `sandboxed`, `lease`, `leases`, `handoff`, `handoffs`, `hand-off`, `hand-offs`, `cloud session`, `cloud sessions`, `remote session`, `remote sessions`

Fragments (rejected anywhere in a line, even inside a longer word or path): `/home/user`, `claude-0/`, `/root/.`, `.ccr/`, `claude.ai/code`, `session_01`

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

The verifier sees only the candidate diff, in a fresh snapshot. It does not see your commands, output, or reasoning. Evidence for a behavior criterion must cite commands the verifier can run itself in that snapshot; your own git status and git diff runs do not count as evidence. Commit hashes made during verification do not exist in the real history, so never pin or cite a commit hash made during verification in code, docs, or design-revision text (BRIEF.md included); describe the content instead. The one exception is the pinned instance: when the brief has a "Pinned source" section, cite its url, ref and SHA as given.

## Report

End with the producer report as your structured output.
