# Job 20261013-093553-j4he: Complete requirements map, strategy view, roadmap; cite by heading

## Request (verbatim)

> Retry 20261013-091253-vgjz. Second time line cites into CHANGELOG.md drifted because the same change edits it: don't cite line numbers in any file this change edits; cite a heading or quote instead.

## Reading (accepted by the user)

Redo the failed retry: deliver the full BRIEF.md requirements map plus a strategy view and a roadmap, not a sample table. Files this change edits (CHANGELOG.md, the map, new docs) are cited by heading or quote, never by line number; untouched files such as BRIEF.md may keep file:line cites.

Assumptions:
- The tree holds only the earlier sample table; the failed attempt's changes were discarded.
- Strategy view and roadmap go in separate docs under docs/design/, linked from the map.
- Citing BRIEF.md by line is fine because this change does not edit it.

## Acceptance criteria

1. [content] docs/design/requirements-map.md covers every requirement in BRIEF.md as rows with IDs, each marked covered, partial, or not covered, replacing the sample table.
2. [content] No file:line or file:range citation points into any file this change edits (CHANGELOG.md, the map, any new or edited doc); those are cited by heading or short quote.
3. [content] Line cites into untouched files (BRIEF.md, docs/records.md, etc.) match the tree, and the map states which files may be cited by line and why.
4. [content] A strategy view (requirements grouped by goal with coverage status) and a roadmap (ordered next steps for partial and uncovered rows, linked to row IDs) exist and are linked from the map.
5. [content] The CHANGELOG.md Unreleased entry describes the map, strategy view and roadmap, says rows quote or paraphrase the requirement (matching the map's Wording section), and has no line-number cites.
6. [behavior] Run git status and git diff --stat: only Markdown under docs/ and CHANGELOG.md (and possibly README.md) changed. Run cargo fmt --check to confirm nothing else is affected.

## Expected deliverables

- docs/design/requirements-map.md (modify)
- docs/design/strategy-view.md (create)
- docs/design/roadmap.md (create)
- CHANGELOG.md (modify)
- README.md (modify)

## Out of scope

- Code, config, or governance file changes
- Editing BRIEF.md or records/
- Implementing any roadmap item

## User defaults that apply

- evidence: Claims cite their sources, and numbers are measured or labeled as estimates. Here: Every row cites a source; cites into edited files use headings or quotes so they cannot drift.
- strangers: Code, commits, docs, and records are written for an outside reader. Here: Docs are written for an outside reader.
- small: Prefer one focused change; size is a design choice, not a wish list. Here: Docs-only change.
- verified: Done means independently verified, and user-facing behavior is verified by exercising it. Here: Verifier checks that cites match the tree.

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
