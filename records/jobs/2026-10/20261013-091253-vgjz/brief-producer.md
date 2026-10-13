# Job 20261013-091253-vgjz: Expand requirements map to full BRIEF coverage; add strategy view, roadm

## Request (verbatim)

> Retry 20261013-080904-m4yj: it landed only a sample table; the request was the full requirements map plus the strategy view and roadmap.

## Reading (accepted by the user)

Replace the 8-row sample in docs/design/requirements-map.md with a map covering every BRIEF.md requirement section (O1–O5, A1–A13, S1–S7, D1–D3, standards, addenda), each with cited evidence and a coverage grade. Add a strategy view (what is answered, partial, open, grouped by theme) and a roadmap ordering the gaps, in the same doc or a sibling doc, plus a changelog entry.

Assumptions:
- The strategy view and roadmap go in new docs under docs/design/ (strategy-view.md, roadmap.md) linked from the map; no README change.
- Full means every numbered requirement heading in BRIEF.md (O1-O5, A1-A13, S1-S7, D1-D3, plus Standards, Boundaries and dated addenda) has at least one row, including rows graded Open where nothing answers it.
- Coverage grades keep the existing Direct/Partial vocabulary and add Open for unanswered requirements.
- Roadmap items are ordered by evidence in the repo (gaps found in the map), not invented new scope; no code changes.

## Acceptance criteria

1. [content] docs/design/requirements-map.md has at least one row for every numbered requirement heading in BRIEF.md (O1-O5, A1-A13, S1-S7, D1-D3) and for Standards and each dated addendum; it no longer describes itself as a sample.
2. [content] Each row has a BRIEF.md line citation whose number matches the quoted requirement and evidence with file:line citations, and a coverage grade of Direct, Partial or Open.
3. [content] A strategy view groups the rows by theme and states counts of Direct, Partial and Open that match the map's rows.
4. [content] A roadmap lists the Partial and Open items in a stated order with a rationale for the ordering, each linked to its map row.
5. [content] CHANGELOG.md has an entry under Unreleased that describes the map as covering all requirement sections, plus the strategy view and roadmap, without overstating.
6. [content] No secrets or personal data appear in the new docs, and no files other than docs and CHANGELOG.md change.
7. [behavior] Run cargo test and cargo clippy from the repository root; both pass, since any doc-checking tests still hold.

## Expected deliverables

- docs/design/requirements-map.md (modify)
- docs/design/strategy-view.md (create)
- docs/design/roadmap.md (create)
- CHANGELOG.md (modify)
- README.md (modify)

## Out of scope

- Code changes to close any gap the map finds
- Changes to BRIEF.md, records/, or governance files
- README.md edits
- Verifying that requirements are met in full beyond citing evidence

## User defaults that apply

- evidence: Claims cite their sources, and numbers are measured or labeled as estimates. Here: Every row cites file:line evidence quoted from the repo; unmeasured claims are labeled as estimates or Open.
- strangers: Code, commits, docs, and records are written for an outside reader. Here: Docs are written for an outside reader with the line-number caveat and coverage definitions kept.
- small: Prefer one focused change; size is a design choice, not a wish list. Here: Docs only, one focused change: map, strategy view, roadmap, one changelog entry.
- lazy-requests: Requests are short; recover hidden requirements from the repository before asking. Here: Scope of strategy view and roadmap recovered from the failed predecessor's findings and the BRIEF headings.
- verified: Done means independently verified, and user-facing behavior is verified by exercising it. Here: Verifier checks cited lines against quoted text and heading coverage against BRIEF.md.

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
