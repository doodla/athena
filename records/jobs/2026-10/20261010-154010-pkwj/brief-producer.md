# Job 20261010-154010-pkwj: Warn in setup when Cargo's bin directory is not on PATH

## Request (verbatim)

> Have setup say when Cargo's bin directory is not on PATH, so the README's cargo steps work on a clean machine (n6, docs/design/reviews/athena-v0-acceptance-1.md).

## Reading (accepted by the user)

Setup's Ready step will tell a new user when ${CARGO_HOME:-~/.cargo}/bin was not on their PATH and give the export line, so the README's cargo steps work on a clean machine. Because setup.sh prepends that directory to its own PATH, it must also tell xtask whether the caller's PATH already had it.

Assumptions:
- Audience: a new user running ./setup.sh on a clean machine, then following the README's cargo steps.
- xtask cannot see the caller's original PATH because setup.sh prepends the cargo bin directory before exec, so setup.sh must export a marker (e.g. ATHENA_CARGO_BIN_ADDED=1) when it prepends; xtask warns when the marker is set or the directory is absent from PATH.
- The existing ~/.local/bin warning (xtask/src/main.rs ~line 410) is the model to mirror.
- The warning goes in the Ready step so it is the last thing the user sees.

## Acceptance criteria

1. [content] setup.sh (a governance file, changed deliberately) exports a marker whenever it prepends Cargo's bin directory to PATH, in both prepend places, and does not otherwise change behavior.
2. [content] xtask/src/main.rs has a pure helper deciding whether Cargo's bin directory (from CARGO_HOME, else HOME/.cargo) is missing from the caller's PATH, taking the marker into account, with unit tests for present, absent, unset PATH, and marker set.
3. [content] The Ready step prints the directory and an exact `export PATH="<dir>:$PATH"` line when it was not on the caller's PATH, and prints nothing extra when it was.
4. [behavior] Run `cargo test -p xtask`: all tests pass including the new PATH-detection tests.
5. [behavior] Run `cargo clippy --all-targets -- -D warnings` and `cargo fmt --check`: both clean.
6. [content] README setup wording (if touched) matches the new behavior; CHANGELOG notes the change if the repo convention requires it.

## Expected deliverables

- xtask/src/main.rs (modify)
- setup.sh (modify) (a governance file: allowed because the reading lists it)
- README.md (modify)
- CHANGELOG.md (modify)

## Out of scope

- Editing the user's shell profile automatically
- Changing the ~/.local/bin warning
- Changes to rustup installation logic

## User defaults that apply

- rust: All code is Rust. Non-Rust files appear only where a platform requires them, and each is listed with its reason. Here: Change is Rust in xtask plus a minimal shell marker.
- tested: Every module has passing tests; cargo test passes and clippy is clean. Here: Unit tests for the helper; cargo test and clippy clean.
- behavior-evidence: Acceptance criteria must check the requested behavior happening, not that types, files or wiring exist. A protection or denial test first shows the action succeeds without the protection, as the same user and in the same environment the real component runs as (root), then shows the action is refused. It fails, not skips or passes silently, if the environment cannot enforce the protection; a file mode that root bypasses is not proof. Worked examples are in docs/design/defaults-loop.md section 3.7. Here: Criteria check the warning appears or not based on PATH, via tests.
- small: Prefer one focused change; size is a design choice, not a wish list. Here: One focused change in setup output.
- mechanism: Rules that matter are enforced by mechanism, or explicitly marked prose-only. Here: Detection is code with unit tests, not prose.
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
