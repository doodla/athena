# Job 20261013-052503-8rak: Retry check-skipping for prose-only changes without breaking the gate

## Request (verbatim)

> Retry 20261013-050804-e6cj.

## Reading (accepted by the user)

Re-do the earlier job (retry of 045315-qaph) that lets a job's checks be skipped when the change touches only a named set of paths (for example docs or studies JSON). The last attempt failed because athena.toml gained `skip_when_only` in the same commit as the parser change, and the gate parsed it with the old parser. This reading lands the parser, runner, tests and docs, and leaves athena.toml untouched.

Assumptions:
- The failure was a bootstrap problem: the gate judges a commit with the parent's parser, so a new athena.toml key cannot land in the same commit as the parser that reads it.
- I cannot see the original request (045315-qaph). I infer it from the failure and the verifier notes: skip checks for docs-only or studies-JSON-only changes, with tests named a_studies_json_only_change_runs_every_check and a docs-only test, and a row M31 in docs/enforcement.md.
- The feature is inert until a follow-up edits athena.toml. This is the smallest change that passes the gate.

## Acceptance criteria

1. [content] CheckConfig in crates/athena-data/src/config.rs accepts an optional `skip_when_only` list. An absent list means the check always runs. Existing configurations parse unchanged.
2. [content] athena.toml is NOT modified. It must contain no `skip_when_only` key, so the gate's old parser accepts the landing commit. A note (docs or CHANGELOG) says enabling the key is a separate follow-up job after this one lands.
3. [content] The check runner skips a check only when every changed path matches a named set in the configuration. An empty change list or any non-matching path runs the check. The skip is recorded in the run output with a note, not silently.
4. [content] The path-matching rules (including the `*.ext` suffix form matching at any depth) are documented in code comments or docs/enforcement.md.
5. [content] docs/enforcement.md has a row for the new mechanism that names its tests.
6. [behavior] Run `cargo test --locked`. All tests pass, including new unit tests for config parsing, path matching, and skip versus run decisions. Run `cargo clippy --all-targets --locked -- -D warnings` and `cargo fmt --all --check`. Both are clean.

## Expected deliverables

- crates/athena-data/src/config.rs (modify)
- crates/athena-core/src/capability.rs (modify)
- crates/athena-core/src/checks.rs (modify)
- crates/athena-cap-change/src/lib.rs (modify)
- crates/athena-cap-study/src/lib.rs (modify)
- crates/athena-core/src/run_pin_tests.rs (modify)
- docs/enforcement.md (modify)
- CHANGELOG.md (modify)
- docs/decisions.md (modify)
- crates/athena-cli/tests/common/mod.rs (modify)
- athena.toml (modify) (a governance file: allowed because the reading lists it)

## Out of scope

- Adding `skip_when_only` or any new key to athena.toml (follow-up job)
- Changing the gate's parser or the history check
- Changing records/ or the owner's files
- Adding new classes of work

## User defaults that apply

- tested: Every module has passing tests; cargo test passes and clippy is clean. Here: New unit tests for config parsing, path matching, and the skip decision. cargo test and clippy must stay clean.
- mechanism: Rules that matter are enforced by mechanism, or explicitly marked prose-only. Here: The skip rule is enforced in code and gets a row in docs/enforcement.md.
- small: Prefer one focused change; size is a design choice, not a wish list. Here: Parser, runner, and docs only. athena.toml stays untouched to avoid the bootstrap failure.
- lazy-requests: Requests are short; recover hidden requirements from the repository before asking. Here: Recovered the intent from the referenced job's failure text and the repository.
- strangers: Code, commits, docs, and records are written for an outside reader. Here: Docs and comments explain the path-matching rules for an outside reader.

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
