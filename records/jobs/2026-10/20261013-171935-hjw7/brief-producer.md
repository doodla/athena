# Job 20261013-171935-hjw7: Compact reader rendering of defaults; add 'root-cause' default

## Request (verbatim)

> Retry 20261013-170708-3p3y with the verifier's major finding fixed: leave the existing defaults' statements exactly as they are (no rewording into extra sentences); only the new 'root-cause' entry and the compact rendering change. Mention the compact rendering in the CHANGELOG entry.

## Reading (accepted by the user)

Change DefaultsModel::render so the reader sees only the first sentence of each statement, keeping full statements for producer/verifier briefs. Add a multi-sentence 'root-cause' entry to defaults.toml and leave every existing statement byte-for-byte unchanged; note the compact rendering in the CHANGELOG.

Assumptions:
- 'Compact' means first sentence of each statement, split on '. ' or terminal period, so single-sentence statements render unchanged.
- The edited design doc is docs/design/athena-v0.md (the verifier's nit), if a design edit is made at all.
- The earlier candidate reached 1723 of 1800 tokens; leaving existing statements whole adds about 40 tokens (estimate), still under budget.
- Sentence splitting must not break on abbreviations like 'e.g.'; the root-cause statement is worded to avoid them.

## Acceptance criteria

1. [content] Every existing default in defaults.toml (rust, strangers, evidence, buy-first, tested, public, small, verified, designed, lazy-requests, mechanism) has its statement, applies_to and source byte-identical to the base commit (git diff shows only added lines for the new entry).
2. [content] defaults.toml gains a 'root-cause' entry: fix where the fault originates, not the symptom or the one instance that surfaced; name an out-of-scope cause and propose the fix rather than working around it; the change description names the root cause and why the change addresses it; the root-cause fix is the one focused change ('small'). Source cites BRIEF.md addendum '2026-10-13: Root causes'. Its first sentence alone conveys the core rule.
3. [content] No new defaults.toml field is introduced (the installed gate still parses the file), and BRIEF.md and the token budgets in athena.toml are unchanged.
4. [content] DefaultsModel::render (reader form) emits only the first sentence of each statement; full statements remain available to the producer/verifier briefs via get(). Statements containing ';' or ', and' remain whole in the reader form because they are one sentence.
5. [content] CHANGELOG.md has an entry that adds the 'root-cause' default and mentions the compact reader rendering.
6. [behavior] Run `cargo test` and `cargo clippy --all-targets` and `cargo fmt --check`: all pass, including a new unit test that render() shows only the first sentence of a multi-sentence statement, keeps a semicolon statement whole, and that the reader's defaults component is within its token budget in crates/athena-cli/tests/budgets.rs.

## Expected deliverables

- crates/athena-data/src/defaults.rs (modify)
- defaults.toml (modify) (a governance file: allowed because the reading lists it)
- CHANGELOG.md (modify)
- docs/design/athena-v0.md (modify)
- crates/athena-core/src/reading.rs (modify)
- crates/athena-cli/tests/budgets.rs (modify)

## Out of scope

- Rewording any existing default's statement
- Raising token budgets or editing athena.toml
- Editing BRIEF.md
- Adding a new defaults.toml field
- Changing other defaults' meaning

## User defaults that apply

- rust: All code is Rust. Non-Rust files appear only where a platform requires them, and each is listed with its reason. Here: Compact rendering is implemented in Rust in athena-data.
- tested: Every module has passing tests; cargo test passes and clippy is clean. Here: Add unit tests for render; cargo test and clippy must pass.
- small: Prefer one focused change; size is a design choice, not a wish list. Here: One focused change: compact render plus the single new entry.
- mechanism: Rules that matter are enforced by mechanism, or explicitly marked prose-only. Here: Token budget test enforces the reader's size limit.
- verified: Done means independently verified, and user-facing behavior is verified by exercising it. Here: Verifier compares defaults.toml diff and runs cargo checks.
- strangers: Code, commits, docs, and records are written for an outside reader. Here: CHANGELOG and doc comments written for an outside reader.
- public: Everything committed is public: no secrets, credentials, or personal data, and transcripts are redacted. Here: No secrets or personal data in the change.

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
