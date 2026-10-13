# Job 20261013-170708-3p3y: Give the reader compact defaults; add the root-cause default

## Request (verbatim)

> Fix the root cause of three failed tries at adding the 'root-cause' default (69gf, t1y6): the reader's always-loaded set carries every default's full statement, so it is at 1799 of 1800 tokens and any new default must cut meaning to fit. The reader only needs enough to choose which defaults apply; the full statements matter in the producer and verifier briefs, which carry only the named defaults. Give the reader a compact form of each default and keep full statements for the briefs, then add the 'root-cause' default with its full meaning: fix where the fault originates, not the symptom or the one instance that surfaced; name an out-of-scope cause and propose the fix rather than working around it; the change's description names the root cause and why the change addresses it; relation to 'small': the root-cause fix is the one focused change. Its source is the BRIEF.md addendum '2026-10-13: Root causes'. Do not raise budgets or edit BRIEF.md. If a new defaults.toml field is needed, mind that the installed gate must still parse the file in the same commit.

## Reading (accepted by the user)

The reader's render() will show each default's id, classes and only the first sentence of its statement, while producer and verifier briefs keep the full statement. Then add the 'root-cause' default with its full meaning, with no new defaults.toml field, because the installed gate rejects unknown fields.

Assumptions:
- The installed gate parses defaults.toml with the old structs, which use deny_unknown_fields (crates/athena-data/src/defaults.rs; gate content.rs:89). A new field would fail it, so I add none.
- The compact form is derived in code: the statement up to its first sentence end, with the rest only in briefs. Existing statements are mostly one sentence, so they are unchanged.
- The root-cause statement starts with a one-sentence gist such as 'Fix where the fault originates, not the symptom or the one instance that surfaced.' Later sentences carry the out-of-scope-cause rule, the description rule, and the relation to 'small'.
- The root-cause entry applies_to is code, config, build, docs, tests, prompts, design (or '*'), with source 'BRIEF.md 2026-10-13: Root causes'. I cannot read BRIEF.md here.
- Token budgets in athena.toml are not touched. The reader set should fall well under 1800 once the new entry is added.

## Acceptance criteria

1. [content] defaults.toml has a new [[default]] id 'root-cause' with a full statement covering: fix where the fault originates, not the symptom or the one instance; name an out-of-scope cause and propose the fix instead of working around it; the change description names the root cause and why the change addresses it; the root-cause fix is the one focused change ('small'). Its source cites BRIEF.md '2026-10-13: Root causes'.
2. [content] defaults.toml adds no new field, and DefaultsModel and Default keep their fields (deny_unknown_fields intact), so the installed gate parses the file.
3. [content] DefaultsModel::render() emits a compact form of each default (id, classes, first sentence of the statement). The briefs code in crates/athena-context/src/briefs.rs still uses the full statement.
4. [behavior] Run cargo test -p athena-data and cargo test -p athena-context. New tests pass: render drops statement text after the first sentence for a multi-sentence default, the briefs keep the full statement, and the repo defaults.toml parses and contains 'root-cause'.
5. [behavior] Run cargo test, cargo clippy --all-targets and cargo fmt --check. All pass, including existing budget tests that load defaults.toml.
6. [content] athena.toml budgets and BRIEF.md are unchanged.

## Expected deliverables

- defaults.toml (modify) (a governance file: allowed because the reading lists it)
- crates/athena-data/src/defaults.rs (modify)
- crates/athena-context/src/briefs.rs (modify)
- crates/athena-core/src/reading.rs (modify)
- CHANGELOG.md (modify)
- docs/design.md (modify)
- crates/athena-cli/tests/budgets.rs (modify)

## Out of scope

- Raising any token budget in athena.toml
- Editing BRIEF.md
- Adding a new defaults.toml field or changing the parse schema
- Changing other defaults' meaning
- Changes to the producer or verifier brief layout

## User defaults that apply

- rust: All code is Rust. Non-Rust files appear only where a platform requires them, and each is listed with its reason. Here: All changes are Rust and TOML; no new non-Rust files.
- tested: Every module has passing tests; cargo test passes and clippy is clean. Here: Add unit tests for compact rendering; cargo test and clippy must pass.
- small: Prefer one focused change; size is a design choice, not a wish list. Here: One focused change: compact reader form plus the new default.
- mechanism: Rules that matter are enforced by mechanism, or explicitly marked prose-only. Here: A test enforces that render is compact and briefs stay full.
- strangers: Code, commits, docs, and records are written for an outside reader. Here: Doc comments explain the compact rule for an outside reader.
- public: Everything committed is public: no secrets, credentials, or personal data, and transcripts are redacted. Here: No secrets or personal data in the change.
- verified: Done means independently verified, and user-facing behavior is verified by exercising it. Here: The verifier runs cargo test, clippy and fmt --check.
- lazy-requests: Requests are short; recover hidden requirements from the repository before asking. Here: I recovered the schema and gate constraints from the repository.

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
