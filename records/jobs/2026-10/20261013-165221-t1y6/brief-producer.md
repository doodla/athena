# Job 20261013-165221-t1y6: Add short root-cause default within the reader token budget

## Request (verbatim)

> Retry 20261013-164814-69gf: the 'root-cause' default failed the test every_role_set_is_within_its_budget (reader set about 1969 tokens against a budget of 1800; the test says pay for additions with removals). Keep the same meaning but make the entry short, and if it still does not fit, tighten wording elsewhere in the reader's always-loaded set rather than raising the budget. Run cargo test before handing in.

## Reading (accepted by the user)

Add the 'root-cause' entry to defaults.toml (source: BRIEF.md "2026-10-13: Root causes") in one or two compact sentences. Keep its meaning: fix the origin, report out-of-scope causes, name the cause in the description, and tie it to 'small'. If the reader set still exceeds 1800 tokens, tighten wording in the reader's always-loaded text and leave the budget alone.

Assumptions:
- The failed job's change is not in this checkout, so the entry is re-added from scratch.
- The ~169 token overage is mostly the long entry; a short entry plus light trimming of the reader frame should fit.
- The defaults.toml version stays 1 unless a test requires a bump.

## Acceptance criteria

1. [content] defaults.toml has a [[default]] with id "root-cause", source "BRIEF.md 2026-10-13: Root causes", and applies_to ["*"] or a justified list.
2. [content] The statement covers: fix where the fault originates, not the symptom or one instance; name an out-of-scope cause and propose the fix rather than working around it; the fix's description names the root cause and why the change addresses it.
3. [content] The statement says how it relates to 'small': the root-cause fix is the one focused change, and an out-of-scope cause is reported, not fixed.
4. [content] The entry is short (about 400 bytes of statement or less). BRIEF.md and the budget constants are unchanged.
5. [behavior] Run `cargo test --locked`: every test passes, including budgets::every_role_set_is_within_its_budget (reader set at or under 1800 tokens).
6. [behavior] Run `cargo clippy` and `cargo fmt --check`: both are clean.

## Expected deliverables

- defaults.toml (modify) (a governance file: allowed because the reading lists it)
- crates/athena-context/prompts/reader.md (modify)
- crates/athena-context/prompts/contract.md (modify)
- CHANGELOG.md (modify)

## Out of scope

- Editing BRIEF.md
- Raising any token budget
- Changing other defaults' meaning
- Changing the budget estimator or tests

## User defaults that apply

- small: Prefer one focused change; size is a design choice, not a wish list. Here: One focused change: one short default, plus only the wording trims needed to fit the budget.
- tested: Every module has passing tests; cargo test passes and clippy is clean. Here: cargo test, clippy, and fmt must pass.
- mechanism: Rules that matter are enforced by mechanism, or explicitly marked prose-only. Here: The token budget test enforces the limit; the budget stays fixed.
- evidence: Claims cite their sources, and numbers are measured or labeled as estimates. Here: The entry cites its BRIEF.md addendum as source.
- verified: Done means independently verified, and user-facing behavior is verified by exercising it. Here: The verifier runs cargo test.

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
