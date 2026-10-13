# Job 20261013-164814-69gf: Add 'root-cause' default to defaults.toml

## Request (verbatim)

> Add a 'root-cause' entry to defaults.toml whose source is the BRIEF.md addendum '2026-10-13: Root causes' (already committed; do not edit BRIEF.md). Statement: when a request reports a failure, find where the fault originates and fix it there, not the symptom or the one instance that surfaced; when the root cause lies outside the request's scope, name it and propose the fix instead of silently working around it; a fix's description names the root cause and why the change addresses it, so the verifier can check it. Say how it relates to 'small': the root-cause fix is the one focused change, and an out-of-scope cause is reported, not fixed.

## Reading (accepted by the user)

Append one [[default]] entry, id "root-cause", to defaults.toml, sourced to "BRIEF.md 2026-10-13: Root causes". Its statement covers fixing at the origin, naming out-of-scope causes with a proposed fix, describing the root cause in the fix, and how this relates to 'small'. No BRIEF.md edit and no code change.

Assumptions:
- applies_to is ["*"], because failure reports can concern code, tests, docs, config or builds.
- The entry is appended at the end of the file and the file's version stays 1, since entries are added without changing the schema.
- The statement is one string of a few sentences, in the style of the other entries, though longer.
- docs/decisions.md line 634 ('eleven defaults') is a historical decision record and is left alone.

## Acceptance criteria

1. [content] defaults.toml has exactly one new [[default]] entry with id = "root-cause", placed after the existing entries, with the same four keys (id, statement, applies_to, source) as the others.
2. [content] The source field cites the BRIEF.md addendum by name, e.g. "BRIEF.md 2026-10-13: Root causes".
3. [content] The statement says: when a request reports a failure, find where the fault originates and fix it there, not the symptom or the one instance that surfaced.
4. [content] The statement says: when the root cause lies outside the request's scope, name it and propose the fix instead of silently working around it.
5. [content] The statement says: a fix's description names the root cause and why the change addresses it, so the verifier can check it.
6. [content] The statement says how it relates to 'small': the root-cause fix is the one focused change, and an out-of-scope cause is reported, not fixed.
7. [content] BRIEF.md and the existing eleven defaults entries are unchanged, and the version value stays 1 unless the change gives a reason to bump it.
8. [behavior] Run `cargo test -p athena-data`: repository_defaults_parse passes, so the file parses with no duplicate ids. Run `cargo test` and `cargo clippy`: both pass, since several crates embed defaults.toml with include_str!.

## Expected deliverables

- defaults.toml (modify) (a governance file: allowed because the reading lists it)
- CHANGELOG.md (modify)
- docs/decisions.md (modify)

## Out of scope

- Editing BRIEF.md
- Changing prompts, briefs or code so the verifier checks root-cause descriptions mechanically
- Updating the historical 'eleven defaults' line in docs/decisions.md
- Changing the existing 'small' entry's text

## User defaults that apply

- small: Prefer one focused change; size is a design choice, not a wish list. Here: One entry in one governance file, with no code or doc changes beyond it.
- strangers: Code, commits, docs, and records are written for an outside reader. Here: The statement is plain enough for an outside reader to follow without context.
- evidence: Claims cite their sources, and numbers are measured or labeled as estimates. Here: The source field cites the BRIEF.md addendum.
- mechanism: Rules that matter are enforced by mechanism, or explicitly marked prose-only. Here: The existing parse test guards the file's validity. The rule itself is prose, delivered to briefs by the defaults renderer.
- tested: Every module has passing tests; cargo test passes and clippy is clean. Here: cargo test and clippy must still pass.
- verified: Done means independently verified, and user-facing behavior is verified by exercising it. Here: The verifier runs cargo test to confirm the file parses.
- lazy-requests: Requests are short; recover hidden requirements from the repository before asking. Here: The id, source and statement points were all given, and the file layout was recovered from the repository.

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
