# Job 20261014-003651-xq5m: Build D87/D88 backtest with fixtures made in temp dirs, no JSON files

## Request (verbatim)

> Retry 20261014-002531-pazf: same build. Its only failure was the policy test every_tracked_file_is_rust_or_a_listed_kind, which refused the committed JSON fixture files under crates/athena-cli/tests/fixtures/backtest/. Build the fixture records inside the Rust tests in a temporary directory instead of committing JSON files.

## Reading (accepted by the user)

Redo job 20261014-002531-pazf's read-only backtest (typed D87/D88 header, supersession parsing, records/ reader, exact-match signal, one introspection record for the earliest D87 event), as in section 5 of the decision-sets review. The tests write their fixture records into a temporary directory at run time, so no JSON fixtures are committed and the policy test every_tracked_file_is_rust_or_a_listed_kind passes.

Assumptions:
- The repository holds none of the earlier job's work (grep for 'backtest' finds only two design docs), so this is a full rebuild with the one fixture change.
- The backtest lives in athena-cli, going by the fixture path in the failure output.
- Using std::env::temp_dir with a unique subdirectory, or an existing dev-dependency such as tempfile if the workspace already has one, is acceptable (buy-first).
- Real records/ are not available to tests, so the tests verify pass and fail logic on synthetic records.

## Acceptance criteria

1. [content] No JSON or other non-Rust fixture files are added under crates/athena-cli/tests/fixtures/ or elsewhere; git status shows only Rust, docs and manifest changes.
2. [content] Backtest tests build their records/ fixtures (jobs outcomes, interventions) in a temporary directory created inside the test, and remove it afterwards.
3. [content] The backtest reads records/ in Athena code (not through an agent), parses the supersession phrases of docs/decisions.md, defines the typed header for D87 and D88 only, runs one exact-match signal, and emits one introspection record for the earliest matching event naming D87.
4. [content] The code reports pass or fail by the review's section 5 conditions, and a fail is reported as a finding with its reason, not forced to pass.
5. [behavior] Run `cargo test --locked`: all tests pass, including xtask's policy test every_tracked_file_is_rust_or_a_listed_kind and the new backtest tests.
6. [behavior] Run `cargo clippy --all-targets --locked` and `cargo fmt --check`: both clean.
7. [content] No Jev, no brief change, and no governance file is modified.

## Expected deliverables

- crates/athena-cli/src/backtest.rs (create)
- crates/athena-cli/src/lib.rs (modify)
- crates/athena-cli/src/main.rs (modify)
- crates/athena-cli/tests/backtest.rs (create)
- crates/athena-cli/Cargo.toml (modify)
- Cargo.lock (modify)
- CHANGELOG.md (modify)
- docs/design/decision-sets.md (modify)

## Out of scope

- Jev or any brief change
- Weakening or extending the every_tracked_file_is_rust_or_a_listed_kind policy list
- Typed headers for decisions other than D87 and D88
- Changing records/

## User defaults that apply

- rust: All code is Rust. Non-Rust files appear only where a platform requires them, and each is listed with its reason. Here: All new code and fixtures are Rust.
- tested: Every module has passing tests; cargo test passes and clippy is clean. Here: New module has passing tests, with clippy clean.
- root-cause: Fix the root cause, not the symptom or the one instance that surfaced. The fault is fixed where it originates; when that lies outside the job's scope, name it and propose the fix rather than working around it. The change description names the root cause and why the change addresses it. The root-cause fix is the one focused change. Here: The fixtures move out of tracked files so the policy test is not weakened.
- small: Prefer one focused change; size is a design choice, not a wish list. Here: One focused change that rebuilds the same backtest.
- public: Everything committed is public: no secrets, credentials, or personal data, and transcripts are redacted. Here: Fixture records contain no personal data or secrets.
- verified: Done means independently verified, and user-facing behavior is verified by exercising it. Here: Full cargo test, clippy and fmt --check are run.

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
