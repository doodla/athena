# Job 20261014-002531-pazf: Add read-only D87 decision backtest command with fixture tests

## Request (verbatim)

> Build the first build named in section 5 of docs/design/reviews/decision-sets-review-1.md, as revised in docs/design/decision-sets.md and docs/design/defaults-loop.md: a read-only backtest of one decision, with no Jev and no brief change. Write the typed header for D87 and D88 only; parse the supersession phrases of docs/decisions.md; read records/ in Athena code (not through an agent) for the D87-era landings and interventions; run one exact-match signal over them; produce one introspection record for the earliest matching event naming D87. The pass and fail conditions are those in the review's section 5. Report the actual result either way: a fail is a finding that closes the build with its reason, not something to make pass.

## Reading (accepted by the user)

Add a read-only Rust backtest: typed headers for D87 and D88 only, a parser for the supersession phrases in docs/decisions.md, and code that reads records/ and runs one exact-match signal. Agent copies lack records/, so this job delivers the command and tests on small fixture records. The real run, introspection record and findings report come after landing.

Assumptions:
- The command is a new athena-cli subcommand (clap) with its logic in a library module, probably athena-data or athena-cli, chosen by where record readers already live.
- Fixture records are small files under a test fixtures directory; they are not copies of real records.
- The parser runs against the real docs/decisions.md in tests, since that file is present in agent copies; the D46->D79 and D87->D88 hits are asserted there.
- The introspection record is written by the command only when the signal matches; no model call is made in this change, so the record's language-model drafting step is out of scope.
- No new dependencies unless a crate already in the workspace lacks what is needed.

## Acceptance criteria

1. [content] A typed header exists for D87 and D88 only, with kind, scope, status, enforcement, assumptions and signals. D87's enforcement is not code-check, and D87 is marked amended_by D88.
2. [behavior] Run cargo test. It passes, including a test that the supersession parser over docs/decisions.md returns the D46->D79 and D87->D88 hits plus the Amends entries, matched case-insensitively, with no entry lacking a target id.
3. [behavior] Run cargo clippy --all-targets and cargo fmt --check. Both are clean.
4. [behavior] Fixture tests show that over fixture records with at least 2 distinct jobs having matching hand-change interventions, the signal marks D87 under-review and cites the job ids and intervention ids. Over a fixture with 1 job, or with an unrelated decision, it marks nothing. D88 is never marked.
5. [behavior] Run the command with cargo run against the fixture records directory. It prints the D87-era landing and intervention ids found and the single signal's verdict. The code makes no agent, model or gateway call, and it reads records/ only.
6. [content] The command is read-only apart from an introspection record written when the signal matches. Its decisions_matched field holds D87, and the code checks that every earlier_cases id exists and is in the same class.
7. [content] The change does not edit docs/decisions.md, defaults.toml, any brief, athena.toml or other governance files, or any Jev or gateway code. The real run over records/, the real introspection record and the findings report are stated as out of this change.

## Expected deliverables

- crates/athena-cli/src/backtest.rs (create)
- crates/athena-cli/src/main.rs (modify)
- crates/athena-cli/src/lib.rs (modify)
- crates/athena-cli/Cargo.toml (modify)
- crates/athena-core/src/decision_header.rs (create)
- crates/athena-core/src/lib.rs (modify)
- crates/athena-cli/tests/backtest.rs (create)
- Cargo.lock (modify)
- CHANGELOG.md (modify)
- README.md (modify)

## Out of scope

- Running the backtest over the real records/ directory
- The real introspection record and the findings report, which come after landing
- Any language-model or gateway call
- Jev, brief, defaults.toml and docs/decisions.md changes
- Typed headers for decisions other than D87 and D88
- New record kinds other than the typed header and the introspection record

## User defaults that apply

- rust: All code is Rust. Non-Rust files appear only where a platform requires them, and each is listed with its reason. Here: All new code and tests are Rust.
- tested: Every module has passing tests; cargo test passes and clippy is clean. Here: Unit tests for the header, the parser, the record reader and the signal on fixtures; cargo test passes and clippy is clean.
- small: Prefer one focused change; size is a design choice, not a wish list. Here: One command covering only D87 and D88; no Jev, brief or gateway change.
- verified: Done means independently verified, and user-facing behavior is verified by exercising it. Here: The verifier runs the command with cargo run against the fixtures.
- strangers: Code, commits, docs, and records are written for an outside reader. Here: Module docs and the command help are written for an outside reader.
- root-cause: Fix the root cause, not the symptom or the one instance that surfaced. The fault is fixed where it originates; when that lies outside the job's scope, name it and propose the fix rather than working around it. The change description names the root cause and why the change addresses it. The root-cause fix is the one focused change. Here: The signal is exact-match and is not tuned; a fail is reported as a finding.
- buy-first: Adopt good, maintained tools and crates before building; build only for needs they don't meet, and record the call. Here: Use existing workspace crates (serde, clap) rather than new parsing libraries.

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
