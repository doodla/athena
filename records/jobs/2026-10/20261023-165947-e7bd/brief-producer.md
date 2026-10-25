# Job 20261023-165947-e7bd: Restore records-history design and accepted decision D111

## Request (verbatim)

> Bring the accepted records-history design onto this branch. Jobs 20261023-052543-9bw7 and 20261023-155649-fy1x wrote and verified it, but on a base that main later reverted, so they were abandoned. Their final text is on the branch archive/next4-2026-10-23 (read it with git show archive/next4-2026-10-23:<path>). Restore docs/design/records-history.md from there, and add its accepted decision entry (archived as D112) and its two CHANGELOG entries. Number the decision with the next free number in docs/decisions.md (D111 here) and update every reference to D112 in the design to match. Leave out everything the archive holds for the spec job (D111 there, docs/spec.toml, docs/ontology.toml, docs/enforcement.md). Otherwise keep the text as archived, except where it describes state that differs on this branch. Only docs/design/records-history.md, docs/decisions.md and CHANGELOG.md change.

## Reading (accepted by the user)

Re-land the archived records-history design (docs/design/records-history.md), its accepted decision entry renumbered from D112 to D111, and its two CHANGELOG entries, taken from archive/next4-2026-10-23. Spec-job material (archived D111, spec.toml, ontology.toml, enforcement.md) is left out. The audience is Sri and the verifier, who compare the text with the archive.

Assumptions:
- The archive branch is readable with git show, and its design text is final (verified twice).
- D111 is free here: decisions.md currently ends at D110.
- Differences in branch state are limited to things like decision numbers, the CHANGELOG position, and references to files or decisions that were left out (the spec job's).

## Acceptance criteria

1. [content] docs/design/records-history.md matches `git show archive/next4-2026-10-23:docs/design/records-history.md` except that every D112 is now D111 and any state that differs on this branch is adjusted; the diff against the archive shows only those changes.
2. [content] docs/decisions.md has a new entry headed D111 (the next free number after D110, placed after it), with the archived D112 text, status accepted, citing the BRIEF.md addenda of 2026-10-23. It has no D112 entry and does not contain the archived spec job's D111.
3. [content] The design contains no remaining reference to D112, and each D111 reference in it points to the records-history decision.
4. [content] CHANGELOG.md has the archive's two entries for the records-history design and decision under [Unreleased]. Their wording is unchanged except that D112 becomes D111. There are no entries for the spec job.
5. [content] Only docs/design/records-history.md, docs/decisions.md and CHANGELOG.md change. docs/spec.toml, docs/ontology.toml and docs/enforcement.md are absent or unchanged.
6. [behavior] Run `git diff --stat` and `git status` to confirm that exactly the three paths changed. Run `git diff` to check the D111 renumbering and the absence of spec-job content.

## Expected deliverables

- docs/design/records-history.md (create)
- docs/decisions.md (modify)
- CHANGELOG.md (modify)

## Out of scope

- Restoring the spec job's D111, docs/spec.toml, docs/ontology.toml or docs/enforcement.md
- Any code, athena.toml or workflow changes
- Implementing the records branch or the build plan

## User defaults that apply

- small: Prefer one focused change; size is a design choice, not a wish list. Here: Docs-only change limited to three files.
- verified: Done means independently verified, and user-facing behavior is verified by exercising it. Here: Independent diff comparison against the archive.
- strangers: Code, commits, docs, and records are written for an outside reader. Here: Text kept as archived, written for outside readers.
- outcome-check: The outcome is the check behind the work, not the activity done. For audience-facing quality, a fresh model reader that took no part in the work rates the live result as a newcomer, and a rating below 4 of 5 blocks completion. Prose-only: no loop code computes the rating yet. Here: Criteria compare the final text against the archive and check the numbering.
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
