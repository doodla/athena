# Job 20261013-095538-59x4: Reorder roadmap so acceleration work comes first, study alongside

## Request (verbatim)

> Reorder the roadmap: Sri's 2026-10-13 addendum makes acceleration Athena's mandate, so speed work (sccache's 0 hits, parallel jobs incl. shared CHANGELOG/decisions collisions, native PR landing, timings) goes first, with the firstmate study running alongside.

## Reading (accepted by the user)

Edit docs/design/roadmap.md so a new first acceleration step (sccache hit rate, parallel jobs with shared CHANGELOG/decisions collisions, native PR landing, timings fed back) precedes the others, citing the BRIEF.md 2026-10-13 "Athena owns acceleration" addendum. The firstmate study step (RM-6) is marked as running alongside it. Docs only, no code.

Assumptions:
- The 'acceleration' addendum is BRIEF.md 'Athena owns acceleration' dated 2026-10-13 (lines 494-496); BRIEF.md itself is not edited.
- The sccache '0 hits' figure comes from the user, not from anything measured in the repository; the roadmap will cite it as reported by the user.
- Existing RM-n identifiers stay stable where possible to avoid breaking references; a new step is added at the top and RM-8 and AD-11 rows move into it.
- Nothing is implemented by this change; the roadmap remains a proposal, as it states.

## Acceptance criteria

1. [content] docs/design/roadmap.md opens its ordered steps with an acceleration step that cites the BRIEF.md 2026-10-13 'Athena owns acceleration' addendum (BRIEF.md:494-496).
2. [content] The acceleration step covers: sccache showing 0 cache hits (diagnose and fix), parallel jobs including collisions on shared CHANGELOG.md and docs/decisions.md, native pull-request landing, and timings fed back into the system; each with a 'done when' test.
3. [content] The firstmate study step (formerly RM-6) is stated to run alongside the acceleration step, not after it, and the 'How the order was chosen' paragraph is rewritten to explain the new order.
4. [content] Every partial or uncovered row still appears in exactly one step; rows moved from RM-8 (A4-3, A5-3, A5-4, A5-5, SD-6, AD-4, AD-5, AD-9) and AD-11 are not duplicated, and the stated step count and row total are correct.
5. [content] Any references to step numbers or 'first three steps' in strategy-view.md, requirements-map.md and README.md are consistent with the new order; CHANGELOG.md has an entry for the reorder.
6. [behavior] Run `git diff --stat` and `git status` to confirm only documentation files changed; run `cargo test` to confirm nothing else broke.

## Expected deliverables

- docs/design/roadmap.md (modify)
- CHANGELOG.md (modify)
- docs/design/strategy-view.md (modify)
- docs/decisions.md (modify)
- docs/design/requirements-map.md (modify)

## Out of scope

- Implementing any acceleration work (sccache fix, parallel jobs, PR landing, timing feedback).
- Editing BRIEF.md.
- Running or changing the firstmate study.
- Changing athena.toml or CI configuration.

## User defaults that apply

- strangers: Code, commits, docs, and records are written for an outside reader. Here: The reordered roadmap explains the reason for the order to an outside reader.
- evidence: Claims cite their sources, and numbers are measured or labeled as estimates. Here: The new step cites BRIEF.md line numbers and labels the 0-hit sccache figure as user-reported, not measured.
- small: Prefer one focused change; size is a design choice, not a wish list. Here: Docs-only reorder; no implementation of the speed work.
- lazy-requests: Requests are short; recover hidden requirements from the repository before asking. Here: Recovered the step content from roadmap.md RM-8, AD-11 and decisions D79-D81.

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
