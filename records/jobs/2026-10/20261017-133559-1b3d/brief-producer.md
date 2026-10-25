# Job 20261017-133559-1b3d: Exempt squash 39962ed from the history check in athena.toml

## Request (verbatim)

> Main is red since 12:55 UTC: CI's 'the full history' check fails on commit 39962ed02fec9da2d87cd4a3e287121ed229cd0f (PR #130), the squash that carried both job 20261010-154004-60r4 (its PR #129 was closed unmerged) and job 20261017-121143-sg0j; it reports a broken chain between them and that neither landing is valid in that commit. Add 39962ed02fec9da2d87cd4a3e287121ed229cd0f to [history] exempt in athena.toml with an explaining comment, the same remedy as ee4b0dd (D66) and f2a8453 (D70). athena.toml is the only deliverable besides the changelog.

## Reading (accepted by the user)

Main's 'full history' check fails on 39962ed (PR #130 squash carrying two jobs, one from closed PR #129). Add that commit id to [history] exempt in athena.toml with an explaining comment, as for ee4b0dd and f2a8453, and note it in the changelog.

Assumptions:
- Goal: turn main green for maintainers and CI by exempting the one commit; audience is the maintainer reading athena.toml.
- The exempt list rule that the commit must already be in history is satisfied because 39962ed is on main.
- Comment follows the existing style of the [history] block, citing no new decision number unless one is assigned; the changelog entry is added under Unreleased.

## Acceptance criteria

1. [content] athena.toml [history] exempt contains the full id 39962ed02fec9da2d87cd4a3e287121ed229cd0f alongside the five existing entries, none removed or altered.
2. [content] A comment above exempt explains the entry for a maintainer: the squash of PR #130 carried jobs 20261010-154004-60r4 (PR #129 closed unmerged) and 20261017-121143-sg0j, so the chain between them is broken and neither landing is valid in that commit; same remedy as ee4b0dd (D66) and f2a8453 (D70).
3. [content] CHANGELOG.md under [Unreleased] has an entry saying 39962ed is exempted from the history check and why.
4. [behavior] Run cargo test and cargo clippy: both pass, so the config still parses and any config-validating tests accept the new entry. Run cargo run -- check --history if available; it should no longer fail on 39962ed (records may be absent in the snapshot).
5. [content] No file other than athena.toml and CHANGELOG.md changes; no record under records/ is edited.

## Expected deliverables

- athena.toml (modify) (a governance file: allowed because the reading lists it)
- CHANGELOG.md (modify)

## Out of scope

- Fixing the chain-landing check so squashes of multiple jobs pass
- Editing any record or job
- Code changes in crates/

## User defaults that apply

- small: Prefer one focused change; size is a design choice, not a wish list. Here: Single one-line config change plus changelog.
- public: Everything committed is public: no secrets, credentials, or personal data, and transcripts are redacted. Here: Only commit ids and job ids are added; no personal data.
- verified: Done means independently verified, and user-facing behavior is verified by exercising it. Here: Behavior criterion runs cargo test/clippy.
- strangers: Code, commits, docs, and records are written for an outside reader. Here: Comment explains the exemption for an outside reader.
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
