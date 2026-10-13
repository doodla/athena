# Job 20261013-080904-m4yj: Add requirements map doc with correct line citations, plus changelog ent

## Request (verbatim)

> Retry 20261013-080006-zgry.

## Reading (accepted by the user)

Retry of a job that creates docs/design/requirements-map.md (mapping BRIEF.md requirements to evidence) and a CHANGELOG.md entry. The last attempt failed on wrong CHANGELOG line citations (off by four after the entry was added), a mis-ranged S5 citation, and a missing BRIEF.md:102 'Restarting' row; fix all three.

Assumptions:
- The original request (074419-sczw) is not visible; I infer the deliverable from the failed verification: docs/design/requirements-map.md plus a 4-line CHANGELOG.md entry near the top.
- The file does not exist in the current tree, so it is created from scratch.
- Line citations refer to files in the final (post-change) tree, as the doc's section 1 states.

## Acceptance criteria

1. [content] docs/design/requirements-map.md exists and states that line numbers refer to files in the same tree.
2. [content] Every CHANGELOG.md line citation in the map matches the final CHANGELOG.md, including the '## [0.1.0]' heading and 'The speedup is not measured.' after the new entry is added.
3. [content] The S5 citation points to the exact lines of the bullet it describes (the crate bullet containing 'eight evidence rules that no waiver clears').
4. [content] BRIEF.md:102 ('Restarting: ...') has its own row in the map, not only indirect coverage.
5. [content] Every other BRIEF.md citation in the map matches the quoted text at that line.
6. [content] CHANGELOG.md has a new entry for the map and no other changes.
7. [behavior] Run `git diff` and `git status`: only docs/design/requirements-map.md and CHANGELOG.md changed; no governance files touched.
8. [behavior] Run `cargo fmt --check`, `cargo clippy`, and `cargo test`: all pass unchanged.

## Expected deliverables

- docs/design/requirements-map.md (create)
- CHANGELOG.md (modify)
- README.md (modify)

## Out of scope

- Code changes
- Changes to BRIEF.md
- Governance files
- records/

## User defaults that apply

- evidence: Claims cite their sources, and numbers are measured or labeled as estimates. Here: Every claim in the map cites file and line, checked against the final tree.
- strangers: Code, commits, docs, and records are written for an outside reader. Here: Doc written for an outside reader.
- small: Prefer one focused change; size is a design choice, not a wish list. Here: One doc plus one changelog entry.
- lazy-requests: Requests are short; recover hidden requirements from the repository before asking. Here: Recovered intent from the prior verification findings.
- verified: Done means independently verified, and user-facing behavior is verified by exercising it. Here: Citations are checked against the final tree by the verifier.

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
