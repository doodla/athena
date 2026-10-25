# Job 20261013-164013-16zf: Add root-cause default and 2026-10-13 BRIEF addendum

## Request (verbatim)

> Make favoring root-cause fixes a standing default. Add a BRIEF.md addendum dated 2026-10-13 that quotes Sri verbatim: 'Have Athena favor fixing root causes'. Add a defaults.toml entry citing it: when a request reports a failure, find where the fault originates and fix it there, rather than patching the symptom or the one instance that surfaced; when the root cause lies outside the request's scope, name it and propose the fix instead of silently working around it. Say how it relates to the existing 'small' default. The verifier must be able to check it: a fix's description names the root cause and why the change addresses it.

## Reading (accepted by the user)

Append a dated 2026-10-13 addendum to BRIEF.md quoting Sri verbatim ('Have Athena favor fixing root causes'), and add a `root-cause` entry to defaults.toml citing it. The entry says to fix where the fault originates, name and propose out-of-scope root causes, relate to `small`, and require fix descriptions to name the root cause and why the change addresses it.

Assumptions:
- Version stays at 1: this is a new entry, not a change to an existing one. Version bumping is not asked for.
- The new default's id is 'root-cause'.
- The verifiability requirement goes in the statement, so the verifier's brief carries it. Only defaults the reader names reach the producer and verifier.
- Addendum format follows earlier ones: a '### 2026-10-13: Title' heading plus a bullet quoting Sri.
- No Rust code change is needed because defaults.rs parses entries generically.

## Acceptance criteria

1. [content] BRIEF.md ends the Addenda section with a new '### 2026-10-13: ...' heading, after the existing 'Athena owns acceleration' addendum and without altering earlier text, quoting Sri verbatim: 'Have Athena favor fixing root causes'.
2. [content] defaults.toml gains one [[default]] with a kebab-case id (e.g. 'root-cause'), a source naming the new BRIEF.md 2026-10-13 addendum, and a sensible applies_to (e.g. code, tests, config, build, docs).
3. [content] The statement says: when a request reports a failure, find where the fault originates and fix it there, not the symptom or the one instance that surfaced; when the root cause is outside the request's scope, name it and propose the fix instead of silently working around it.
4. [content] The entry (statement or a comment beside it) says how it relates to 'small': the root-cause fix is the one focused change; widening scope is not allowed, so out-of-scope causes are reported, not fixed. The 'small' entry is unchanged.
5. [content] The entry makes the rule checkable: a fix's description must name the root cause and say why the change addresses it.
6. [behavior] Run `cargo test` and confirm it passes. This includes the defaults.toml parse tests and the budget tests that include defaults.toml. Run `cargo clippy` and `cargo fmt --check` and confirm both are clean.
7. [content] No files other than the listed deliverables change; records/ is untouched.

## Expected deliverables

- BRIEF.md (modify)
- defaults.toml (modify) (a governance file: allowed because the reading lists it)
- CHANGELOG.md (modify)
- docs/decisions.md (modify)
- crates/athena-data/src/defaults.rs (modify)

## Out of scope

- Changing prompts or code to enforce the rule mechanically
- Editing the existing 'small' default
- Bumping defaults version
- Retroactively reviewing past fixes

## User defaults that apply

- small: Prefer one focused change; size is a design choice, not a wish list. Here: One focused change: two governance/spec files edited, nothing else.
- strangers: Code, commits, docs, and records are written for an outside reader. Here: The entry's wording is plain and self-contained for an outside reader.
- evidence: Claims cite their sources, and numbers are measured or labeled as estimates. Here: The entry cites the BRIEF.md addendum as its source, and the quote is verbatim.
- public: Everything committed is public: no secrets, credentials, or personal data, and transcripts are redacted. Here: The addendum names Sri only as the existing addenda do, with no email or personal data.
- mechanism: Rules that matter are enforced by mechanism, or explicitly marked prose-only. Here: The rule is prose-only for now; the entry or the summary should say so, with the verifier check as the enforcement point.
- verified: Done means independently verified, and user-facing behavior is verified by exercising it. Here: cargo test, clippy and fmt --check confirm the files still parse and the budgets hold.

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
