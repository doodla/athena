# Job 20261014-064441-jatw: Update requirements map and strategy view to current state

## Request (verbatim)

> Bring docs/design/requirements-map.md and docs/design/strategy-view.md up to date. Add rows for the BRIEF.md addenda added since the map was written (2026-10-13: Root causes; Introspection on every correction; Decision sets, and knowing when a decision is not working; 2026-10-14: Common sense over rigid rules), and re-assess the status of every row against what has landed since (native PR landing with squash resync and merge via REST, the root-cause and changelog defaults, compact reader defaults, the pipe-deadlock fix, decision sets with the typed decision view, host-error records and the D87 backtest, introspection with retry-from-record, defaults proposals, the Jev client in shadow and score-jev, the non-Rust intent check). Each status change cites the commit or file that justifies it. Recount the coverage table exactly. Rewrite 'What to do next' as an ordered list toward the brief, naming what blocks each step.

## Reading (accepted by the user)

Add four addenda rows (AD-12 to AD-15) to the requirements map, re-assess every row against landed work with a cited commit or file for each status change, recount the strategy view's coverage table (124 rows become 128), and rewrite "What to do next" as an ordered list with blockers. Docs only.

Assumptions:
- The new rows are numbered AD-12 to AD-15 in addendum order, giving 128 rows in total, up from 124.
- Commit hashes come from git log of the working repository; the reader could not see history, so the producer must look them up.
- Row statuses are judged by reading the cited files, not by running commands, consistent with the map's existing method.
- roadmap.md is touched only if the new 'What to do next' contradicts it; the request names only two files.
- CHANGELOG.md is listed because this repository's change reading lists it as a deliverable by default.

## Acceptance criteria

1. [content] requirements-map.md has four new rows in the Addenda table, AD-12 Root causes, AD-13 Introspection on every correction, AD-14 Decision sets and knowing when a decision is not working, AD-15 Common sense over rigid rules, each citing the correct BRIEF.md line range (BRIEF.md:498-513) and evidence files.
2. [content] Every row whose status changed from the current map (for example AD-4, AD-5, A5-5, A6-4, A2-4, A1-1, A8-1, AD-10, AD-11, A4-2, JG-5) shows the new status and cites a commit hash (checked with git log) or a file that justifies it; rows with unchanged status are left as they are or re-confirmed.
3. [content] Evidence cells still cite BRIEF by line and other files by heading or quote, per the map's 'How to cite' rules, and the map's intro, statuses and wording sections remain accurate (for example the claim that no command measured coverage).
4. [content] strategy-view.md per-goal Covered/Partial/Not covered lists match the map's statuses exactly, each ID appears under exactly one goal (the four new rows are assigned to goals), and the coverage table counts per goal and the Total row equal the counts of rows in the map's tables.
5. [content] strategy-view.md 'What to do next' is an ordered (numbered) list toward the brief, and each step names what blocks it; it is consistent with docs/design/roadmap.md or the roadmap is updated to agree.
6. [content] CHANGELOG.md Unreleased gets an entry for this documentation update.
7. [behavior] Run `git diff --stat` and `git status`: only the listed docs (and CHANGELOG.md) changed; run `cargo test` and `cargo fmt --check` to confirm docs-only changes break no doc-link or boundary tests.

## Expected deliverables

- docs/design/requirements-map.md (modify)
- docs/design/strategy-view.md (modify)
- CHANGELOG.md (modify)
- docs/design/roadmap.md (modify)

## Out of scope

- BRIEF.md is not edited.
- No code, tests or configuration change.
- No new designs; designs not yet built keep their not-covered or partial status.
- Rewriting the roadmap beyond keeping it consistent.

## User defaults that apply

- evidence: Claims cite their sources, and numbers are measured or labeled as estimates. Here: Each status change cites a commit or file; the map still says no command measured coverage.
- strangers: Code, commits, docs, and records are written for an outside reader. Here: Plain cell text that an outside reader can follow.
- small: Prefer one focused change; size is a design choice, not a wish list. Here: One focused docs change, limited to the map and the strategy view plus the changelog.
- lazy-requests: Requests are short; recover hidden requirements from the repository before asking. Here: Row IDs, counts and statuses were recovered from the existing tables, not asked about.
- root-cause: Fix the root cause, not the symptom or the one instance that surfaced. The fault is fixed where it originates; when that lies outside the job's scope, name it and propose the fix rather than working around it. The change description names the root cause and why the change addresses it. The root-cause fix is the one focused change. Here: The coverage table is recounted from the tables themselves, not patched by adding to old numbers.
- changelog: (not in the defaults model) Here: BRIEF.md asks for tagged releases and a changelog, so the change records itself in CHANGELOG.md.

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
